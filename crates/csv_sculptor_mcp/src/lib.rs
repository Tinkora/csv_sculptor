use csv_sculptor_core::{
    CoreError, CsvTable, FilterCondition, FilterOp, count_formula_like_cells, detect_delimiter,
    filter, parse_csv, sort, to_csv, to_json, to_markdown_table, to_sql_insert, to_tsv, to_yaml,
};
use rmcp::{
    Json, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{Implementation, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt};

const MAX_INPUT_BYTES: usize = csv_sculptor_core::parse::MAX_INPUT_BYTES;

pub const SCHEMA_VERSION: &str = "1";
pub const MAX_STDIO_MESSAGE_BYTES: usize = 64 * 1024 * 1024;

#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct ParseRequest {
    #[schemars(description = "UTF-8 CSV/TSV input, limited to 10 MiB")]
    pub input: String,
    #[serde(default = "default_has_header")]
    pub has_header: bool,
}

fn default_has_header() -> bool {
    true
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct TableInput {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub delimiter: char,
    pub row_count: usize,
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct FilterRequest {
    pub table: TableInput,
    pub conditions: Vec<FilterConditionInput>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct FilterConditionInput {
    pub column: String,
    pub operator: FilterOperator,
    #[serde(default)]
    pub value: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub enum FilterOperator {
    Equals,
    NotEquals,
    Contains,
    StartsWith,
    EndsWith,
    GreaterThan,
    LessThan,
    IsEmpty,
    IsNotEmpty,
}

impl From<FilterOperator> for FilterOp {
    fn from(operator: FilterOperator) -> Self {
        match operator {
            FilterOperator::Equals => Self::Equals,
            FilterOperator::NotEquals => Self::NotEquals,
            FilterOperator::Contains => Self::Contains,
            FilterOperator::StartsWith => Self::StartsWith,
            FilterOperator::EndsWith => Self::EndsWith,
            FilterOperator::GreaterThan => Self::GreaterThan,
            FilterOperator::LessThan => Self::LessThan,
            FilterOperator::IsEmpty => Self::IsEmpty,
            FilterOperator::IsNotEmpty => Self::IsNotEmpty,
        }
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct SortRequest {
    pub table: TableInput,
    pub column: String,
    #[serde(default = "default_ascending")]
    pub ascending: bool,
}

fn default_ascending() -> bool {
    true
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct ExportRequest {
    pub table: TableInput,
    pub format: ExportFormat,
    #[serde(default = "default_table_name")]
    pub table_name: String,
}

fn default_table_name() -> String {
    "table_name".to_owned()
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Json,
    Yaml,
    Markdown,
    Sql,
    Csv,
    Tsv,
}

#[derive(Clone, Debug, PartialEq, Serialize, JsonSchema)]
pub struct TableOutput {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub delimiter: char,
    pub row_count: usize,
}

impl From<CsvTable> for TableOutput {
    fn from(table: CsvTable) -> Self {
        Self {
            headers: table.headers,
            rows: table.rows,
            delimiter: table.delimiter,
            row_count: table.row_count,
        }
    }
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct Envelope<T> {
    pub schema_version: String,
    pub tool: String,
    pub data: T,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct ParseData {
    pub table: TableOutput,
    pub formula_like_cell_count: usize,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct TableData {
    pub table: TableOutput,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct ExportData {
    pub format: ExportFormat,
    pub content: String,
    pub formula_like_cell_count: usize,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct DelimiterData {
    pub delimiter: String,
    pub delimiter_name: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields)]
pub struct DelimiterRequest {
    pub input: String,
}

#[derive(Debug)]
struct ToolFailure {
    code: &'static str,
    message: String,
}

impl ToolFailure {
    fn invalid_input(message: impl Into<String>) -> Self {
        Self {
            code: "INVALID_INPUT",
            message: message.into(),
        }
    }
}

impl From<CoreError> for ToolFailure {
    fn from(error: CoreError) -> Self {
        Self {
            code: error.code(),
            message: error.to_string(),
        }
    }
}

impl fmt::Display for ToolFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

fn core_error_message(error: CoreError) -> String {
    ToolFailure::from(error).to_string()
}

fn envelope<T>(tool: &str, data: T) -> Envelope<T> {
    Envelope {
        schema_version: SCHEMA_VERSION.to_owned(),
        tool: tool.to_owned(),
        data,
    }
}

fn validate_table(input: TableInput) -> Result<CsvTable, ToolFailure> {
    if input.headers.is_empty() {
        return Err(ToolFailure::invalid_input("table must contain headers"));
    }
    if input.row_count != input.rows.len() {
        return Err(ToolFailure::invalid_input(
            "row_count must equal the number of rows",
        ));
    }
    if input.headers.iter().any(|header| header.trim().is_empty()) {
        return Err(ToolFailure::invalid_input("header names cannot be blank"));
    }
    let mut headers = HashSet::with_capacity(input.headers.len());
    if input.headers.iter().any(|header| !headers.insert(header)) {
        return Err(ToolFailure::invalid_input("header names must be unique"));
    }
    if input
        .rows
        .iter()
        .any(|row| row.len() != input.headers.len())
    {
        return Err(ToolFailure::invalid_input(
            "every row must contain one value per header",
        ));
    }
    if !matches!(input.delimiter, ',' | '\t' | '|' | ';') {
        return Err(ToolFailure::invalid_input(
            "delimiter must be comma, tab, pipe, or semicolon",
        ));
    }

    let data_bytes = input
        .headers
        .iter()
        .chain(input.rows.iter().flatten())
        .map(String::len)
        .sum::<usize>();
    if data_bytes > MAX_INPUT_BYTES {
        return Err(ToolFailure {
            code: "INPUT_TOO_LARGE",
            message: format!("table data exceeds the {MAX_INPUT_BYTES}-byte limit"),
        });
    }

    Ok(CsvTable {
        headers: input.headers,
        rows: input.rows,
        delimiter: input.delimiter,
        row_count: input.row_count,
    })
}

pub fn execute_parse(request: ParseRequest) -> Result<Envelope<ParseData>, String> {
    let table = parse_csv(&request.input, request.has_header).map_err(core_error_message)?;
    Ok(envelope(
        "csv_sculptor_parse",
        ParseData {
            formula_like_cell_count: count_formula_like_cells(&table),
            table: table.into(),
        },
    ))
}

pub fn execute_filter(request: FilterRequest) -> Result<Envelope<TableData>, String> {
    let table = validate_table(request.table).map_err(|error| error.to_string())?;
    let conditions = request
        .conditions
        .into_iter()
        .map(|condition| FilterCondition {
            column: condition.column,
            operator: condition.operator.into(),
            value: condition.value,
        })
        .collect::<Vec<_>>();
    let result = filter(&table, &conditions).map_err(core_error_message)?;
    Ok(envelope(
        "csv_sculptor_filter",
        TableData {
            table: result.into(),
        },
    ))
}

pub fn execute_sort(request: SortRequest) -> Result<Envelope<TableData>, String> {
    let table = validate_table(request.table).map_err(|error| error.to_string())?;
    let result = sort(&table, &request.column, request.ascending).map_err(core_error_message)?;
    Ok(envelope(
        "csv_sculptor_sort",
        TableData {
            table: result.into(),
        },
    ))
}

pub fn execute_export(request: ExportRequest) -> Result<Envelope<ExportData>, String> {
    let table = validate_table(request.table).map_err(|error| error.to_string())?;
    let formula_like_cell_count = count_formula_like_cells(&table);
    let content = match request.format {
        ExportFormat::Json => to_json(&table, false),
        ExportFormat::Yaml => to_yaml(&table),
        ExportFormat::Markdown => to_markdown_table(&table),
        ExportFormat::Sql => to_sql_insert(
            &table,
            if request.table_name.is_empty() {
                "table_name"
            } else {
                &request.table_name
            },
        ),
        ExportFormat::Csv => Ok(to_csv(&table)),
        ExportFormat::Tsv => Ok(to_tsv(&table)),
    }
    .map_err(core_error_message)?;

    let mut warnings = Vec::new();
    if formula_like_cell_count > 0 {
        warnings.push(format!(
            "{formula_like_cell_count} cell(s) look like spreadsheet formulas; review before opening in a spreadsheet"
        ));
    }
    if matches!(request.format, ExportFormat::Sql) {
        warnings
            .push("SQL output is text; review the dialect and permissions before execution".into());
    }

    Ok(envelope(
        "csv_sculptor_export",
        ExportData {
            format: request.format,
            content,
            formula_like_cell_count,
            warnings,
        },
    ))
}

pub fn execute_detect_delimiter(input: &str) -> Result<Envelope<DelimiterData>, String> {
    let delimiter = detect_delimiter(input).map_err(core_error_message)?;
    let delimiter_name = match delimiter {
        ',' => "comma",
        '\t' => "tab",
        '|' => "pipe",
        ';' => "semicolon",
        _ => "unknown",
    };
    Ok(envelope(
        "csv_sculptor_detect_delimiter",
        DelimiterData {
            delimiter: delimiter.to_string(),
            delimiter_name: delimiter_name.to_owned(),
        },
    ))
}

#[derive(Clone)]
pub struct CsvSculptorServer {
    tool_router: ToolRouter<Self>,
}

impl CsvSculptorServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

impl Default for CsvSculptorServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_router(router = tool_router)]
impl CsvSculptorServer {
    #[tool(
        name = "csv_sculptor_parse",
        description = "Parse bounded UTF-8 CSV/TSV text into a structured table."
    )]
    pub fn parse(
        &self,
        Parameters(request): Parameters<ParseRequest>,
    ) -> Result<Json<Envelope<ParseData>>, String> {
        execute_parse(request).map(Json)
    }

    #[tool(
        name = "csv_sculptor_filter",
        description = "Filter a structured table with AND-combined conditions."
    )]
    pub fn filter(
        &self,
        Parameters(request): Parameters<FilterRequest>,
    ) -> Result<Json<Envelope<TableData>>, String> {
        execute_filter(request).map(Json)
    }

    #[tool(
        name = "csv_sculptor_sort",
        description = "Sort a structured table by one column."
    )]
    pub fn sort(
        &self,
        Parameters(request): Parameters<SortRequest>,
    ) -> Result<Json<Envelope<TableData>>, String> {
        execute_sort(request).map(Json)
    }

    #[tool(
        name = "csv_sculptor_export",
        description = "Export a structured table to deterministic text."
    )]
    pub fn export(
        &self,
        Parameters(request): Parameters<ExportRequest>,
    ) -> Result<Json<Envelope<ExportData>>, String> {
        execute_export(request).map(Json)
    }

    #[tool(
        name = "csv_sculptor_detect_delimiter",
        description = "Detect the delimiter used in bounded UTF-8 CSV/TSV text."
    )]
    pub fn detect_delimiter(
        &self,
        Parameters(request): Parameters<DelimiterRequest>,
    ) -> Result<Json<Envelope<DelimiterData>>, String> {
        execute_detect_delimiter(&request.input).map(Json)
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for CsvSculptorServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new(
                "csv_sculptor_mcp",
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions(
                "Process CSV/TSV locally. Input is bounded to 10 MiB and is never uploaded.",
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table() -> TableInput {
        TableInput {
            headers: vec!["name".into(), "age".into()],
            rows: vec![
                vec!["Ada".into(), "37".into()],
                vec!["Bob".into(), "29".into()],
            ],
            delimiter: ',',
            row_count: 2,
        }
    }

    #[test]
    fn parse_returns_versioned_table_envelope() {
        let result = execute_parse(ParseRequest {
            input: "name,age\nAda,37\n".to_owned(),
            has_header: true,
        })
        .expect("parse should succeed");

        assert_eq!(result.schema_version, SCHEMA_VERSION);
        assert_eq!(result.tool, "csv_sculptor_parse");
        assert_eq!(result.data.table.row_count, 1);
        assert_eq!(result.data.table.headers, vec!["name", "age"]);
    }

    #[test]
    fn parse_rejects_input_over_the_core_limit() {
        let input = "x".repeat(MAX_INPUT_BYTES + 1);
        let error = execute_parse(ParseRequest {
            input,
            has_header: true,
        })
        .expect_err("oversized input must be rejected");

        assert!(error.starts_with("INPUT_TOO_LARGE:"));
    }

    #[test]
    fn filter_rejects_jagged_table_before_core_transform() {
        let mut input = table();
        input.rows[0].pop();
        let error = execute_filter(FilterRequest {
            table: input,
            conditions: Vec::new(),
        })
        .expect_err("jagged table must be rejected");

        assert!(error.starts_with("INVALID_INPUT:"));
    }

    #[test]
    fn filter_reports_stable_core_error_codes() {
        let error = execute_filter(FilterRequest {
            table: table(),
            conditions: vec![FilterConditionInput {
                column: "missing".into(),
                operator: FilterOperator::Equals,
                value: "x".into(),
            }],
        })
        .expect_err("unknown columns must be rejected");

        assert!(error.starts_with("COLUMN_NOT_FOUND:"));
    }

    #[test]
    fn filter_and_sort_return_the_expected_rows() {
        let filtered = execute_filter(FilterRequest {
            table: table(),
            conditions: vec![FilterConditionInput {
                column: "age".into(),
                operator: FilterOperator::GreaterThan,
                value: "30".into(),
            }],
        })
        .unwrap();
        assert_eq!(
            filtered.data.table.rows,
            vec![vec!["Ada".to_owned(), "37".to_owned()]]
        );

        let sorted = execute_sort(SortRequest {
            table: table(),
            column: "age".into(),
            ascending: true,
        })
        .unwrap();
        assert_eq!(sorted.data.table.rows[0][0], "Bob");
    }

    #[test]
    fn export_reports_formula_warning_and_format() {
        let mut input = table();
        input.rows[0][0] = "=SUM(1,2)".into();
        let result = execute_export(ExportRequest {
            table: input,
            format: ExportFormat::Csv,
            table_name: default_table_name(),
        })
        .unwrap();

        assert_eq!(result.data.format, ExportFormat::Csv);
        assert_eq!(result.data.formula_like_cell_count, 1);
        assert_eq!(result.data.warnings.len(), 1);
        assert!(result.data.content.contains("=SUM(1,2)"));
    }

    #[test]
    fn delimiter_output_names_tab() {
        let result = execute_detect_delimiter("a\tb\n1\t2").unwrap();
        assert_eq!(result.data.delimiter, "\t");
        assert_eq!(result.data.delimiter_name, "tab");
    }

    #[test]
    fn tool_catalog_exposes_all_five_operations() {
        use rmcp::ServerHandler;

        let server = CsvSculptorServer::new();
        for name in [
            "csv_sculptor_parse",
            "csv_sculptor_filter",
            "csv_sculptor_sort",
            "csv_sculptor_export",
            "csv_sculptor_detect_delimiter",
        ] {
            let tool = server.get_tool(name).expect("tool must be registered");
            assert_eq!(tool.name, name);
            assert_eq!(
                tool.input_schema.get("additionalProperties"),
                Some(&serde_json::json!(false))
            );
            assert!(tool.output_schema.is_some());
        }
    }

    #[test]
    fn request_schemas_reject_unknown_fields() {
        let error = serde_json::from_value::<ParseRequest>(serde_json::json!({
            "input": "a,b\n1,2",
            "unexpected": true
        }))
        .expect_err("unknown request fields must be rejected");

        assert!(error.to_string().contains("unknown field"));
    }
}
