use crate::error::CoreError;
use crate::parse::CsvTable;
use serde::Serialize;
use serde::ser::SerializeMap;

/// Count cells that spreadsheet software may interpret as formulas.
pub fn count_formula_like_cells(table: &CsvTable) -> usize {
    table
        .headers
        .iter()
        .chain(table.rows.iter().flatten())
        .filter(|value| is_formula_like(value))
        .count()
}

fn is_formula_like(value: &str) -> bool {
    matches!(
        value.trim_start_matches(' ').chars().next(),
        Some('=' | '+' | '-' | '@' | '\t' | '\r' | '\n' | '＝' | '＋' | '－' | '＠')
    )
}

struct OrderedRow<'a> {
    headers: &'a [String],
    values: &'a [String],
}

impl Serialize for OrderedRow<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.headers.len()))?;
        for (header, value) in self.headers.iter().zip(self.values) {
            map.serialize_entry(header, value)?;
        }
        map.end()
    }
}

fn ordered_rows(table: &CsvTable) -> Result<Vec<OrderedRow<'_>>, CoreError> {
    if table.headers.is_empty() {
        return Err(CoreError::EmptyTable);
    }
    if table
        .rows
        .iter()
        .any(|row| row.len() != table.headers.len())
    {
        return Err(CoreError::InvalidInput(
            "Every row must contain exactly one value per header".into(),
        ));
    }

    Ok(table
        .rows
        .iter()
        .map(|values| OrderedRow {
            headers: &table.headers,
            values,
        })
        .collect())
}

/// Export the table as a JSON array of objects.
///
/// Each row becomes an object with header keys. If `pretty` is true,
/// the output is pretty-printed with indentation.
pub fn to_json(table: &CsvTable, pretty: bool) -> Result<String, CoreError> {
    let objects = ordered_rows(table)?;

    if pretty {
        serde_json::to_string_pretty(&objects)
            .map_err(|e| CoreError::ExportError(format!("JSON export failed: {e}")))
    } else {
        serde_json::to_string(&objects)
            .map_err(|e| CoreError::ExportError(format!("JSON export failed: {e}")))
    }
}

/// Export the table as a YAML array of objects.
pub fn to_yaml(table: &CsvTable) -> Result<String, CoreError> {
    let objects = ordered_rows(table)?;

    serde_yaml::to_string(&objects)
        .map_err(|e| CoreError::ExportError(format!("YAML export failed: {e}")))
}

/// Export the table as a GitHub-flavored Markdown table.
pub fn to_markdown_table(table: &CsvTable) -> Result<String, CoreError> {
    if table.headers.is_empty() {
        return Err(CoreError::EmptyTable);
    }

    let mut output = String::new();

    // Calculate column widths
    let mut col_widths: Vec<usize> = table.headers.iter().map(|h| h.chars().count()).collect();

    for row in &table.rows {
        for (i, field) in row.iter().enumerate() {
            if i < col_widths.len() {
                let len = field.chars().count();
                if len > col_widths[i] {
                    col_widths[i] = len;
                }
            }
        }
    }

    // Ensure minimum width of 3 for alignment row
    for w in col_widths.iter_mut() {
        if *w < 3 {
            *w = 3;
        }
    }

    // Header row
    output.push('|');
    for (i, header) in table.headers.iter().enumerate() {
        output.push_str(&format!(" {:width$} |", header, width = col_widths[i]));
    }
    output.push('\n');

    // Separator row
    output.push('|');
    for &width in &col_widths {
        output.push_str(&format!(" {:-<width$} |", "", width = width));
    }
    output.push('\n');

    // Data rows
    for row in &table.rows {
        output.push('|');
        for (i, field) in row.iter().enumerate() {
            let escaped = escape_markdown_pipe(field);
            if i < col_widths.len() {
                output.push_str(&format!(" {:width$} |", escaped, width = col_widths[i]));
            }
        }
        output.push('\n');
    }

    Ok(output)
}

/// Escape pipe characters in markdown table cells.
fn escape_markdown_pipe(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', "<br>")
}

/// Export the table as SQL INSERT statements.
///
/// Values are properly quoted and escaped for SQL.
pub fn to_sql_insert(table: &CsvTable, table_name: &str) -> Result<String, CoreError> {
    if table.headers.is_empty() {
        return Err(CoreError::EmptyTable);
    }

    if table.rows.is_empty() {
        return Ok(format!("-- No data to insert into {table_name}\n"));
    }

    let columns: String = table
        .headers
        .iter()
        .map(|h| quote_sql_identifier(h))
        .collect::<Vec<_>>()
        .join(", ");

    let mut output = String::new();

    for row in &table.rows {
        let values: String = row
            .iter()
            .map(|v| format!("'{}'", escape_sql_string(v)))
            .collect::<Vec<_>>()
            .join(", ");

        output.push_str(&format!(
            "INSERT INTO {} ({columns}) VALUES ({values});\n",
            quote_sql_identifier(table_name)
        ));
    }

    Ok(output)
}

/// Escape a string value for SQL insertion.
fn escape_sql_string(s: &str) -> String {
    s.replace('\'', "''")
}

/// Quote a SQL identifier using the portable SQL double-quote form.
fn quote_sql_identifier(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

/// Export the table as CSV (comma-separated).
///
/// Fields containing commas, quotes, or newlines are properly quoted.
pub fn to_csv(table: &CsvTable) -> String {
    let mut output = String::new();

    // Header
    output.push_str(
        &table
            .headers
            .iter()
            .map(|h| csv_quote(h))
            .collect::<Vec<_>>()
            .join(","),
    );
    output.push('\n');

    // Data rows
    for row in &table.rows {
        output.push_str(
            &row.iter()
                .map(|f| csv_quote(f))
                .collect::<Vec<_>>()
                .join(","),
        );
        output.push('\n');
    }

    output
}

/// Export the table as TSV (tab-separated).
///
/// Fields containing tabs or newlines are escaped.
pub fn to_tsv(table: &CsvTable) -> String {
    let mut output = String::new();

    // Header
    output.push_str(&table.headers.join("\t"));
    output.push('\n');

    // Data rows
    for row in &table.rows {
        output.push_str(
            &row.iter()
                .map(|f| f.replace('\t', "\\t").replace('\n', "\\n"))
                .collect::<Vec<_>>()
                .join("\t"),
        );
        output.push('\n');
    }

    output
}

/// Quote a CSV field if it contains special characters.
fn csv_quote(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_table() -> CsvTable {
        CsvTable {
            headers: vec!["name".into(), "age".into(), "city".into()],
            rows: vec![
                vec!["Alice".into(), "30".into(), "NYC".into()],
                vec!["Bob".into(), "25".into(), "LA".into()],
            ],
            delimiter: ',',
            row_count: 2,
        }
    }

    #[test]
    fn test_to_json() {
        let json = to_json(&sample_table(), true).unwrap();
        assert!(json.contains("\"name\": \"Alice\""));
        assert!(json.contains("\"city\": \"LA\""));
        // Verify it's valid JSON
        let _parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn test_to_json_compact() {
        let json = to_json(&sample_table(), false).unwrap();
        assert!(!json.contains('\n'));
        let _parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn test_to_yaml() {
        let yaml = to_yaml(&sample_table()).unwrap();
        assert!(yaml.contains("name: Alice"));
        assert!(yaml.contains("city: LA"));
    }

    #[test]
    fn test_to_markdown_table() {
        let md = to_markdown_table(&sample_table()).unwrap();
        let mut lines = md.lines();
        assert_eq!(lines.next().unwrap().trim_start(), "| name  | age | city |");
        assert!(lines.next().unwrap().contains("---"));
        assert!(lines.any(|line| line.contains("Alice") && line.contains("NYC")));
    }

    #[test]
    fn test_markdown_escapes_pipe() {
        let table = CsvTable {
            headers: vec!["col".into()],
            rows: vec![vec!["a|b".into()]],
            delimiter: ',',
            row_count: 1,
        };
        let md = to_markdown_table(&table).unwrap();
        assert!(md.contains("a\\|b"));
    }

    #[test]
    fn test_to_sql_insert() {
        let sql = to_sql_insert(&sample_table(), "users").unwrap();
        assert!(sql.contains("INSERT INTO \"users\""));
        assert!(sql.contains("VALUES ('Alice', '30', 'NYC')"));
        assert!(sql.contains("VALUES ('Bob', '25', 'LA')"));
    }

    #[test]
    fn test_sql_escapes_quotes() {
        let table = CsvTable {
            headers: vec!["name".into()],
            rows: vec![vec!["O'Brien".into()]],
            delimiter: ',',
            row_count: 1,
        };
        let sql = to_sql_insert(&table, "t").unwrap();
        assert!(sql.contains("O''Brien"));
    }

    #[test]
    fn test_sql_escapes_identifiers() {
        let table = CsvTable {
            headers: vec!["column\"name".into()],
            rows: vec![vec!["value".into()]],
            delimiter: ',',
            row_count: 1,
        };
        let sql = to_sql_insert(&table, "table\"name").unwrap();
        assert!(sql.contains("INSERT INTO \"table\"\"name\""));
        assert!(sql.contains("(\"column\"\"name\")"));
    }

    #[test]
    fn test_to_csv() {
        let csv = to_csv(&sample_table());
        assert!(csv.starts_with("name,age,city\n"));
        assert!(csv.contains("Alice,30,NYC"));
    }

    #[test]
    fn test_to_csv_quotes_special() {
        let table = CsvTable {
            headers: vec!["desc".into()],
            rows: vec![vec!["hello, world".into()]],
            delimiter: ',',
            row_count: 1,
        };
        let csv = to_csv(&table);
        assert!(csv.contains("\"hello, world\""));
    }

    #[test]
    fn test_to_tsv() {
        let tsv = to_tsv(&sample_table());
        assert!(tsv.starts_with("name\tage\tcity\n"));
        assert!(tsv.contains("Alice\t30\tNYC"));
    }
}
