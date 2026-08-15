use crate::error::CoreError;
use crate::export;
use crate::parse::{self, CsvTable};
use crate::transform::{self, FilterCondition};
use serde::Deserialize;
use wasm_bindgen::prelude::*;

/// Serialize a CsvTable to a JsValue via JSON.
///
/// This is the primary bridge format between Rust/WASM and JavaScript.
fn table_to_js(table: &CsvTable) -> Result<JsValue, CoreError> {
    let json =
        serde_json::to_string(table).map_err(|e| CoreError::SerializationError(e.to_string()))?;
    Ok(JsValue::from_str(&json))
}

/// Deserialize a CsvTable from its JSON representation.
fn table_from_js(table_json: &str) -> Result<CsvTable, CoreError> {
    serde_json::from_str(table_json).map_err(|e| CoreError::SerializationError(e.to_string()))
}

/// Parse a CSV/TSV string and return the CsvTable as a JsValue (JSON).
///
/// # Arguments
/// * `input` - Raw CSV/TSV text content
/// * `has_header` - Whether the first row is a header
#[wasm_bindgen]
pub fn wasm_parse_csv(input: &str, has_header: bool) -> Result<JsValue, JsValue> {
    let table = parse::parse_csv(input, has_header).map_err(core_err)?;
    table_to_js(&table).map_err(core_err)
}

/// Apply filter conditions to a serialized CsvTable.
///
/// # Arguments
/// * `table_json` - Serialized CsvTable JSON string
/// * `conditions_json` - JSON array of filter conditions
#[wasm_bindgen]
pub fn wasm_filter(table_json: &str, conditions_json: &str) -> Result<JsValue, JsValue> {
    let table = table_from_js(table_json).map_err(core_err)?;

    #[derive(Deserialize)]
    struct RawCondition {
        column: String,
        operator: String,
        #[serde(default)]
        value: String,
    }

    let raw_conditions: Vec<RawCondition> = serde_json::from_str(conditions_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid conditions JSON: {e}")))?;

    let conditions: Vec<FilterCondition> = raw_conditions
        .into_iter()
        .map(|rc| {
            Ok(FilterCondition {
                column: rc.column,
                operator: rc
                    .operator
                    .parse::<transform::FilterOp>()
                    .map_err(|e| JsValue::from_str(&e.to_string()))?,
                value: rc.value,
            })
        })
        .collect::<Result<_, JsValue>>()?;

    let result = transform::filter(&table, &conditions).map_err(core_err)?;
    table_to_js(&result).map_err(core_err)
}

/// Sort a serialized CsvTable by a column.
///
/// # Arguments
/// * `table_json` - Serialized CsvTable JSON string
/// * `column` - Column name to sort by
/// * `ascending` - Sort direction
#[wasm_bindgen]
pub fn wasm_sort(table_json: &str, column: &str, ascending: bool) -> Result<JsValue, JsValue> {
    let table = table_from_js(table_json).map_err(core_err)?;
    let result = transform::sort(&table, column, ascending).map_err(core_err)?;
    table_to_js(&result).map_err(core_err)
}

/// Select specific columns from a serialized CsvTable.
///
/// # Arguments
/// * `table_json` - Serialized CsvTable JSON string
/// * `columns_json` - JSON array of column name strings
#[wasm_bindgen]
pub fn wasm_select_columns(table_json: &str, columns_json: &str) -> Result<JsValue, JsValue> {
    let table = table_from_js(table_json).map_err(core_err)?;
    let columns: Vec<String> = serde_json::from_str(columns_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid columns JSON: {e}")))?;
    let result = transform::select_columns(&table, &columns).map_err(core_err)?;
    table_to_js(&result).map_err(core_err)
}

/// Deduplicate rows in a serialized CsvTable.
#[wasm_bindgen]
pub fn wasm_deduplicate(table_json: &str) -> Result<JsValue, JsValue> {
    let table = table_from_js(table_json).map_err(core_err)?;
    let result = transform::deduplicate(&table).map_err(core_err)?;
    table_to_js(&result).map_err(core_err)
}

/// Limit the number of rows in a serialized CsvTable.
#[wasm_bindgen]
pub fn wasm_limit(table_json: &str, n: usize) -> Result<JsValue, JsValue> {
    let table = table_from_js(table_json).map_err(core_err)?;
    let result = transform::limit(&table, n);
    table_to_js(&result).map_err(core_err)
}

/// Export a serialized CsvTable to the specified format.
///
/// # Arguments
/// * `table_json` - Serialized CsvTable JSON string
/// * `format` - One of: "json", "json_pretty", "yaml", "markdown", "sql", "csv", "tsv"
/// * `table_name` - Table name for SQL format (optional, defaults to "table_name")
#[wasm_bindgen]
pub fn wasm_export(table_json: &str, format: &str, table_name: &str) -> Result<String, JsValue> {
    let table = table_from_js(table_json).map_err(core_err)?;
    let table_name = if table_name.is_empty() {
        "table_name"
    } else {
        table_name
    };

    match format {
        "json" => export::to_json(&table, false).map_err(core_err),
        "json_pretty" => export::to_json(&table, true).map_err(core_err),
        "yaml" => export::to_yaml(&table).map_err(core_err),
        "markdown" => export::to_markdown_table(&table).map_err(core_err),
        "sql" => export::to_sql_insert(&table, table_name).map_err(core_err),
        "csv" => Ok(export::to_csv(&table)),
        "tsv" => Ok(export::to_tsv(&table)),
        _ => Err(JsValue::from_str(&format!(
            "Unknown export format: {format}"
        ))),
    }
}

/// Count spreadsheet formula-like cells in a serialized CsvTable.
#[wasm_bindgen]
pub fn wasm_formula_like_cell_count(table_json: &str) -> Result<usize, JsValue> {
    let table = table_from_js(table_json).map_err(core_err)?;
    Ok(export::count_formula_like_cells(&table))
}

/// Detect the delimiter character from raw CSV/TSV text.
///
/// Returns a single-character string like ",", "\t", "|", or ";".
#[wasm_bindgen]
pub fn wasm_detect_delimiter(input: &str) -> Result<String, JsValue> {
    let delimiter = parse::detect_delimiter(input).map_err(core_err)?;
    Ok(delimiter.to_string())
}

/// Returns the column headers from a serialized CsvTable as a JSON array.
#[wasm_bindgen]
pub fn wasm_get_headers(table_json: &str) -> Result<JsValue, JsValue> {
    let table = table_from_js(table_json).map_err(core_err)?;
    let json = serde_json::to_string(&table.headers)
        .map_err(|e| core_err(CoreError::SerializationError(e.to_string())))?;
    Ok(JsValue::from_str(&json))
}

// Helper: convert CoreError to JsValue
fn core_err(e: CoreError) -> JsValue {
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(&obj, &"code".into(), &e.code().into()).ok();
    js_sys::Reflect::set(&obj, &"message".into(), &e.to_string().into()).ok();
    obj.into()
}
