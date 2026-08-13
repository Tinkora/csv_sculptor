use crate::error::CoreError;
use crate::parse::CsvTable;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::str::FromStr;

/// Filter operators supported by the transform engine.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum FilterOp {
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

impl FromStr for FilterOp {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Equals" => Ok(Self::Equals),
            "NotEquals" => Ok(Self::NotEquals),
            "Contains" => Ok(Self::Contains),
            "StartsWith" => Ok(Self::StartsWith),
            "EndsWith" => Ok(Self::EndsWith),
            "GreaterThan" => Ok(Self::GreaterThan),
            "LessThan" => Ok(Self::LessThan),
            "IsEmpty" => Ok(Self::IsEmpty),
            "IsNotEmpty" => Ok(Self::IsNotEmpty),
            _ => Err(CoreError::InvalidFilterOperator(s.to_string())),
        }
    }
}

/// A single filter condition targeting a column with an operator and value.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FilterCondition {
    pub column: String,
    pub operator: FilterOp,
    pub value: String,
}

/// Filter rows matching ALL conditions (AND logic).
///
/// Returns a new CsvTable containing only rows that satisfy every condition.
pub fn filter(table: &CsvTable, conditions: &[FilterCondition]) -> Result<CsvTable, CoreError> {
    if table.rows.is_empty() {
        return Ok(table.clone());
    }

    // Validate all columns exist
    for cond in conditions {
        if !table.headers.contains(&cond.column) {
            return Err(CoreError::ColumnNotFound(cond.column.clone()));
        }
    }

    // Build column index lookup
    let col_indices: Vec<usize> = conditions
        .iter()
        .map(|c| table.headers.iter().position(|h| h == &c.column).unwrap())
        .collect();

    let filtered_rows: Vec<Vec<String>> = table
        .rows
        .iter()
        .filter(|row| {
            conditions.iter().enumerate().all(|(i, cond)| {
                let col_idx = col_indices[i];
                let field = row.get(col_idx).map(|s| s.as_str()).unwrap_or("");
                eval_filter(field, &cond.operator, &cond.value)
            })
        })
        .cloned()
        .collect();

    let row_count = filtered_rows.len();

    Ok(CsvTable {
        headers: table.headers.clone(),
        rows: filtered_rows,
        delimiter: table.delimiter,
        row_count,
    })
}

/// Evaluate a single filter condition against a field value.
fn eval_filter(field: &str, op: &FilterOp, value: &str) -> bool {
    match op {
        FilterOp::Equals => field == value,
        FilterOp::NotEquals => field != value,
        FilterOp::Contains => field.to_lowercase().contains(&value.to_lowercase()),
        FilterOp::StartsWith => field.to_lowercase().starts_with(&value.to_lowercase()),
        FilterOp::EndsWith => field.to_lowercase().ends_with(&value.to_lowercase()),
        FilterOp::GreaterThan => {
            // Try numeric comparison first
            if let (Ok(a), Ok(b)) = (field.parse::<f64>(), value.parse::<f64>()) {
                a > b
            } else {
                field > value
            }
        }
        FilterOp::LessThan => {
            if let (Ok(a), Ok(b)) = (field.parse::<f64>(), value.parse::<f64>()) {
                a < b
            } else {
                field < value
            }
        }
        FilterOp::IsEmpty => field.trim().is_empty(),
        FilterOp::IsNotEmpty => !field.trim().is_empty(),
    }
}

/// Sort table rows by a column.
///
/// Attempts numeric sort if all values in the column parse as f64;
/// otherwise falls back to lexicographic (case-insensitive) sort.
pub fn sort(table: &CsvTable, column: &str, ascending: bool) -> Result<CsvTable, CoreError> {
    if table.rows.is_empty() {
        return Ok(table.clone());
    }

    let col_idx = table
        .headers
        .iter()
        .position(|h| h == column)
        .ok_or_else(|| CoreError::ColumnNotFound(column.to_string()))?;

    // Detect if column is numeric
    let is_numeric = table.rows.iter().all(|row| {
        row.get(col_idx)
            .is_some_and(|v| v.trim().parse::<f64>().is_ok())
    });

    let mut sorted_rows = table.rows.clone();

    if is_numeric {
        sorted_rows.sort_by(|a, b| {
            let av = a
                .get(col_idx)
                .and_then(|v| v.trim().parse::<f64>().ok())
                .unwrap_or(f64::NAN);
            let bv = b
                .get(col_idx)
                .and_then(|v| v.trim().parse::<f64>().ok())
                .unwrap_or(f64::NAN);
            let cmp = av.partial_cmp(&bv).unwrap_or(std::cmp::Ordering::Equal);
            if ascending { cmp } else { cmp.reverse() }
        });
    } else {
        sorted_rows.sort_by(|a, b| {
            let av = a.get(col_idx).map(|s| s.to_lowercase()).unwrap_or_default();
            let bv = b.get(col_idx).map(|s| s.to_lowercase()).unwrap_or_default();
            if ascending { av.cmp(&bv) } else { bv.cmp(&av) }
        });
    }

    Ok(CsvTable {
        headers: table.headers.clone(),
        rows: sorted_rows,
        delimiter: table.delimiter,
        row_count: table.row_count,
    })
}

/// Select a subset of columns, preserving their order.
pub fn select_columns(table: &CsvTable, columns: &[String]) -> Result<CsvTable, CoreError> {
    // Validate all columns exist
    let col_indices: Vec<usize> = columns
        .iter()
        .map(|col| {
            table
                .headers
                .iter()
                .position(|h| h == col)
                .ok_or_else(|| CoreError::ColumnNotFound(col.clone()))
        })
        .collect::<Result<_, _>>()?;

    let new_headers: Vec<String> = columns.to_vec();
    let new_rows: Vec<Vec<String>> = table
        .rows
        .iter()
        .map(|row| col_indices.iter().map(|&i| row[i].clone()).collect())
        .collect();
    let row_count = new_rows.len();

    Ok(CsvTable {
        headers: new_headers,
        rows: new_rows,
        delimiter: table.delimiter,
        row_count,
    })
}

/// Remove duplicate rows from the table.
pub fn deduplicate(table: &CsvTable) -> Result<CsvTable, CoreError> {
    if table.rows.is_empty() {
        return Ok(table.clone());
    }

    let mut seen = HashSet::new();
    let mut unique_rows: Vec<Vec<String>> = Vec::new();

    for row in &table.rows {
        // Create a hashable key from the row
        let key: Vec<&str> = row.iter().map(|s| s.as_str()).collect();
        if seen.insert(key) {
            unique_rows.push(row.clone());
        }
    }
    let row_count = unique_rows.len();

    Ok(CsvTable {
        headers: table.headers.clone(),
        rows: unique_rows,
        delimiter: table.delimiter,
        row_count,
    })
}

/// Limit the number of rows to the first `n`.
pub fn limit(table: &CsvTable, n: usize) -> CsvTable {
    let limited_rows: Vec<Vec<String>> = table.rows.iter().take(n).cloned().collect();
    CsvTable {
        headers: table.headers.clone(),
        rows: limited_rows.clone(),
        delimiter: table.delimiter,
        row_count: limited_rows.len(),
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
                vec!["Charlie".into(), "35".into(), "NYC".into()],
                vec!["Diana".into(), "28".into(), "SF".into()],
            ],
            delimiter: ',',
            row_count: 4,
        }
    }

    #[test]
    fn test_filter_equals() {
        let table = sample_table();
        let cond = FilterCondition {
            column: "city".into(),
            operator: FilterOp::Equals,
            value: "NYC".into(),
        };
        let result = filter(&table, &[cond]).unwrap();
        assert_eq!(result.row_count, 2);
        assert_eq!(result.rows[0][0], "Alice");
        assert_eq!(result.rows[1][0], "Charlie");
    }

    #[test]
    fn test_filter_contains() {
        let table = sample_table();
        let cond = FilterCondition {
            column: "name".into(),
            operator: FilterOp::Contains,
            value: "li".into(),
        };
        let result = filter(&table, &[cond]).unwrap();
        assert_eq!(result.row_count, 2); // Alice, Charlie
    }

    #[test]
    fn test_filter_greater_than() {
        let table = sample_table();
        let cond = FilterCondition {
            column: "age".into(),
            operator: FilterOp::GreaterThan,
            value: "28".into(),
        };
        let result = filter(&table, &[cond]).unwrap();
        assert_eq!(result.row_count, 2); // Alice(30), Charlie(35)
    }

    #[test]
    fn test_filter_is_empty() {
        let mut table = sample_table();
        table.rows[0][1] = String::new(); // Alice age empty
        let cond = FilterCondition {
            column: "age".into(),
            operator: FilterOp::IsEmpty,
            value: String::new(),
        };
        let result = filter(&table, &[cond]).unwrap();
        assert_eq!(result.row_count, 1);
        assert_eq!(result.rows[0][0], "Alice");
    }

    #[test]
    fn test_sort_ascending_numeric() {
        let table = sample_table();
        let result = sort(&table, "age", true).unwrap();
        assert_eq!(result.rows[0][0], "Bob"); // 25
        assert_eq!(result.rows[1][0], "Diana"); // 28
        assert_eq!(result.rows[2][0], "Alice"); // 30
        assert_eq!(result.rows[3][0], "Charlie"); // 35
    }

    #[test]
    fn test_sort_descending() {
        let table = sample_table();
        let result = sort(&table, "name", false).unwrap();
        assert_eq!(result.rows[0][0], "Diana");
        assert_eq!(result.rows[3][0], "Alice");
    }

    #[test]
    fn test_select_columns() {
        let table = sample_table();
        let result = select_columns(&table, &["name".into(), "age".into()]).unwrap();
        assert_eq!(result.headers, vec!["name", "age"]);
        assert_eq!(result.rows[0].len(), 2);
        assert_eq!(result.row_count, table.row_count);
    }

    #[test]
    fn test_deduplicate() {
        let mut table = sample_table();
        table
            .rows
            .push(vec!["Alice".into(), "30".into(), "NYC".into()]); // duplicate
        table.row_count = 5;
        let result = deduplicate(&table).unwrap();
        assert_eq!(result.row_count, 4);
        assert_eq!(result.rows.len(), result.row_count);
    }

    #[test]
    fn test_limit() {
        let table = sample_table();
        let result = limit(&table, 2);
        assert_eq!(result.row_count, 2);
        assert_eq!(result.rows.len(), 2);
    }

    #[test]
    fn test_column_not_found() {
        let table = sample_table();
        let cond = FilterCondition {
            column: "unknown".into(),
            operator: FilterOp::Equals,
            value: "x".into(),
        };
        assert!(filter(&table, &[cond]).is_err());
    }
}
