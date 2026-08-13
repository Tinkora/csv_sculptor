use crate::error::CoreError;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Represents a parsed CSV/TSV table with headers and data rows.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CsvTable {
    /// Column names (first row or auto-generated).
    pub headers: Vec<String>,
    /// Data rows, each being a vector of string field values.
    pub rows: Vec<Vec<String>>,
    /// The delimiter character detected or specified.
    pub delimiter: char,
    /// Total number of data rows (excluding header).
    pub row_count: usize,
}

/// Common delimiter candidates in priority order.
const DELIMITER_CANDIDATES: &[char] = &[',', '\t', '|', ';'];

pub const MAX_INPUT_BYTES: usize = 10 * 1024 * 1024;

/// Parse a CSV/TSV string into a CsvTable.
///
/// If `has_header` is true, the first row is treated as column headers.
/// Otherwise, headers are auto-generated as "Column_0", "Column_1", etc.
///
/// The delimiter is auto-detected with the CSV parser so quoted fields do not
/// affect candidate scoring.
pub fn parse_csv(input: &str, has_header: bool) -> Result<CsvTable, CoreError> {
    if input.len() > MAX_INPUT_BYTES {
        return Err(CoreError::InputTooLarge);
    }
    if input.trim().is_empty() {
        return Err(CoreError::InvalidInput(
            "Input is empty or whitespace-only".into(),
        ));
    }

    let delimiter = detect_delimiter(input)?;

    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter as u8)
        .flexible(false)
        .has_headers(has_header)
        .from_reader(input.as_bytes());

    let headers: Vec<String> = if has_header {
        let hdrs = reader
            .headers()
            .map_err(|e| CoreError::ParseError(format!("Failed to read headers: {e}")))?;
        let headers: Vec<String> = hdrs.iter().map(|h| h.to_string()).collect();
        validate_headers(&headers)?;
        headers
    } else {
        // Read the first record to determine column count
        let first_record = reader
            .records()
            .next()
            .transpose()
            .map_err(|e| CoreError::ParseError(format!("Failed to read first record: {e}")))?;

        match first_record {
            Some(record) => {
                let col_count = record.len();
                let headers: Vec<String> = (0..col_count).map(|i| format!("Column_{i}")).collect();

                // Build rows starting from this first record
                let mut rows: Vec<Vec<String>> = Vec::new();
                rows.push(record.iter().map(|f| f.to_string()).collect());

                for result in reader.records() {
                    let record =
                        result.map_err(|e| CoreError::ParseError(format!("Parse error: {e}")))?;
                    rows.push(record.iter().map(|f| f.to_string()).collect());
                }

                let row_count = rows.len();
                return Ok(CsvTable {
                    headers,
                    rows,
                    delimiter,
                    row_count,
                });
            }
            None => {
                return Err(CoreError::InvalidInput(
                    "No data rows found in input".into(),
                ));
            }
        }
    };

    // has_header == true: headers already consumed, now read rows
    let mut rows: Vec<Vec<String>> = Vec::new();

    for result in reader.records() {
        let record = result.map_err(|e| CoreError::ParseError(format!("Parse error: {e}")))?;
        rows.push(record.iter().map(|f| f.to_string()).collect());
    }

    let row_count = rows.len();

    Ok(CsvTable {
        headers,
        rows,
        delimiter,
        row_count,
    })
}

fn validate_headers(headers: &[String]) -> Result<(), CoreError> {
    if headers.iter().any(|header| header.trim().is_empty()) {
        return Err(CoreError::InvalidInput(
            "Header names cannot be blank".into(),
        ));
    }

    let mut seen = HashSet::with_capacity(headers.len());
    for header in headers {
        if !seen.insert(header) {
            return Err(CoreError::InvalidInput(format!(
                "Duplicate header: {header}"
            )));
        }
    }

    Ok(())
}

/// Detect the most likely delimiter from the input text.
///
/// Parses a bounded sample with each supported candidate and prefers the
/// candidate that produces the most consistent multi-column records.
pub fn detect_delimiter(input: &str) -> Result<char, CoreError> {
    if input.trim().is_empty() {
        return Err(CoreError::DelimiterDetectionFailed);
    }

    let delimiter = DELIMITER_CANDIDATES
        .iter()
        .filter_map(|&candidate| delimiter_score(input, candidate).map(|score| (score, candidate)))
        .max_by_key(|(score, _)| *score)
        .map(|(_, delimiter)| delimiter)
        .unwrap_or(',');

    Ok(delimiter)
}

fn delimiter_score(input: &str, delimiter: char) -> Option<(usize, usize, usize)> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter as u8)
        .flexible(true)
        .has_headers(false)
        .from_reader(input.as_bytes());
    let mut widths = Vec::new();

    for record in reader.records().take(10) {
        let record = record.ok()?;
        if !record.is_empty() {
            widths.push(record.len());
        }
    }

    let multi_column_records = widths.iter().filter(|&&width| width > 1).count();
    if multi_column_records == 0 {
        return None;
    }

    let mut frequencies = std::collections::HashMap::new();
    for width in widths {
        if width > 1 {
            *frequencies.entry(width).or_insert(0usize) += 1;
        }
    }
    let (expected_width, consistent_records) = frequencies
        .into_iter()
        .max_by_key(|&(width, count)| (count, width))?;

    Some((multi_column_records, consistent_records, expected_width))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_csv_with_header() {
        let input = "name,age,city\nAlice,30,NYC\nBob,25,LA";
        let table = parse_csv(input, true).unwrap();
        assert_eq!(table.headers, vec!["name", "age", "city"]);
        assert_eq!(table.row_count, 2);
        assert_eq!(table.rows[0], vec!["Alice", "30", "NYC"]);
        assert_eq!(table.delimiter, ',');
    }

    #[test]
    fn test_parse_csv_without_header() {
        let input = "Alice,30,NYC\nBob,25,LA";
        let table = parse_csv(input, false).unwrap();
        assert_eq!(table.headers, vec!["Column_0", "Column_1", "Column_2"]);
        assert_eq!(table.row_count, 2);
    }

    #[test]
    fn test_parse_tsv() {
        let input = "name\tage\tcity\nAlice\t30\tNYC\nBob\t25\tLA";
        let table = parse_csv(input, true).unwrap();
        assert_eq!(table.delimiter, '\t');
        assert_eq!(table.headers, vec!["name", "age", "city"]);
        assert_eq!(table.row_count, 2);
    }

    #[test]
    fn test_parse_pipe_delimited() {
        let input = "name|age|city\nAlice|30|NYC\nBob|25|LA";
        let table = parse_csv(input, true).unwrap();
        assert_eq!(table.delimiter, '|');
        assert_eq!(table.headers, vec!["name", "age", "city"]);
    }

    #[test]
    fn test_parse_semicolon_delimited() {
        let input = "name;age;city\nAlice;30;NYC\nBob;25;LA";
        let table = parse_csv(input, true).unwrap();
        assert_eq!(table.delimiter, ';');
        assert_eq!(table.headers, vec!["name", "age", "city"]);
    }

    #[test]
    fn test_detect_delimiter_comma() {
        let input = "a,b,c\n1,2,3\n4,5,6";
        assert_eq!(detect_delimiter(input).unwrap(), ',');
    }

    #[test]
    fn test_detect_delimiter_tab() {
        let input = "a\tb\tc\n1\t2\t3\n4\t5\t6";
        assert_eq!(detect_delimiter(input).unwrap(), '\t');
    }

    #[test]
    fn test_detect_delimiter_pipe() {
        let input = "a|b|c\n1|2|3\n4|5|6";
        assert_eq!(detect_delimiter(input).unwrap(), '|');
    }

    #[test]
    fn test_empty_input() {
        assert!(parse_csv("", true).is_err());
    }

    #[test]
    fn test_rejects_input_over_10_mib() {
        let input = "x".repeat(10 * 1024 * 1024 + 1);
        assert!(matches!(
            parse_csv(&input, true),
            Err(CoreError::InputTooLarge)
        ));
    }

    #[test]
    fn test_single_column() {
        let input = "name\nAlice\nBob";
        let table = parse_csv(input, true).unwrap();
        assert_eq!(table.headers, vec!["name"]);
        assert_eq!(table.row_count, 2);
    }

    #[test]
    fn test_rejects_jagged_rows() {
        let input = "a,b,c\n1,2\n4,5,6,7";
        assert!(matches!(
            parse_csv(input, true),
            Err(CoreError::ParseError(_))
        ));
    }
}
