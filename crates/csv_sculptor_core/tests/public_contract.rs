use csv_sculptor_core::{
    CoreError, CsvTable, count_formula_like_cells, parse_csv, to_json, to_sql_insert,
};

#[test]
fn delimiter_detection_ignores_delimiters_inside_quoted_fields() {
    let table = parse_csv(
        "\"display,name\";note\n\"Alice, A\";\"uses, several, commas\"\n",
        true,
    )
    .unwrap();

    assert_eq!(table.delimiter, ';');
    assert_eq!(table.headers, ["display,name", "note"]);
    assert_eq!(table.rows[0], ["Alice, A", "uses, several, commas"]);
}

#[test]
fn duplicate_headers_are_rejected_before_object_export() {
    let error = parse_csv("name,name\nAlice,admin\n", true).unwrap_err();

    assert!(matches!(error, CoreError::InvalidInput(_)));
    assert!(error.to_string().contains("Duplicate header"));
}

#[test]
fn blank_headers_are_rejected_before_object_export() {
    let error = parse_csv("name,\nAlice,admin\n", true).unwrap_err();

    assert!(matches!(error, CoreError::InvalidInput(_)));
    assert!(error.to_string().contains("blank"));
}

#[test]
fn inconsistent_row_width_is_rejected_instead_of_truncated() {
    let error = parse_csv("name,role\nAlice,admin,ignored\n", true).unwrap_err();

    assert!(matches!(error, CoreError::ParseError(_)));
}

#[test]
fn field_whitespace_is_preserved() {
    let table = parse_csv("name,note\n Alice ,\" padded \"\n", true).unwrap();

    assert_eq!(table.rows[0], [" Alice ", " padded "]);
}

#[test]
fn json_export_preserves_header_order() {
    let table = parse_csv("z,a,m\n1,2,3\n", true).unwrap();
    let json = to_json(&table, false).unwrap();

    assert_eq!(json, r#"[{"z":"1","a":"2","m":"3"}]"#);
}

#[test]
fn sql_export_preserves_backslashes_and_escapes_quotes() {
    let table = parse_csv("path,note\nC:\\\\Temp,O'Brien\n", true).unwrap();
    let sql = to_sql_insert(&table, "imports").unwrap();

    assert!(sql.contains(r#"'C:\\Temp'"#));
    assert!(!sql.contains(r#"'C:\\\\Temp'"#));
    assert!(sql.contains("'O''Brien'"));
}

#[test]
fn object_export_rejects_malformed_rows() {
    let table = CsvTable {
        headers: vec!["name".into(), "role".into()],
        rows: vec![vec!["Alice".into()]],
        delimiter: ',',
        row_count: 1,
    };

    assert!(matches!(
        to_json(&table, false),
        Err(CoreError::InvalidInput(_))
    ));
}

#[test]
fn formula_warning_covers_documented_prefixes_in_headers_and_rows() {
    let table = CsvTable {
        headers: vec!["＝value".into()],
        rows: vec![
            vec!["=1+1".into()],
            vec!["+1".into()],
            vec!["-1".into()],
            vec!["  @SUM(A1)".into()],
            vec!["\tcommand".into()],
            vec!["\rcommand".into()],
            vec!["\ncommand".into()],
            vec!["＝1+1".into()],
            vec!["＋1".into()],
            vec!["－1".into()],
            vec!["＠command".into()],
            vec!["plain text".into()],
        ],
        delimiter: ',',
        row_count: 12,
    };

    assert_eq!(count_formula_like_cells(&table), 12);
}
