use csv_sculptor_core::wasm::{
    wasm_deduplicate, wasm_detect_delimiter, wasm_export, wasm_filter,
    wasm_formula_like_cell_count, wasm_get_headers, wasm_limit, wasm_parse_csv,
    wasm_select_columns, wasm_sort,
};
use wasm_bindgen::prelude::*;

/// Re-export all core WASM functions for the web target.
/// This crate serves as the thin build target — all logic lives in csv_sculptor_core.

#[wasm_bindgen]
pub fn parse_csv(input: &str, has_header: bool) -> Result<JsValue, JsValue> {
    wasm_parse_csv(input, has_header)
}

#[wasm_bindgen]
pub fn filter_table(table_json: &str, conditions_json: &str) -> Result<JsValue, JsValue> {
    wasm_filter(table_json, conditions_json)
}

#[wasm_bindgen]
pub fn sort_table(table_json: &str, column: &str, ascending: bool) -> Result<JsValue, JsValue> {
    wasm_sort(table_json, column, ascending)
}

#[wasm_bindgen]
pub fn select_columns(table_json: &str, columns_json: &str) -> Result<JsValue, JsValue> {
    wasm_select_columns(table_json, columns_json)
}

#[wasm_bindgen]
pub fn deduplicate_table(table_json: &str) -> Result<JsValue, JsValue> {
    wasm_deduplicate(table_json)
}

#[wasm_bindgen]
pub fn limit_table(table_json: &str, n: usize) -> Result<JsValue, JsValue> {
    wasm_limit(table_json, n)
}

#[wasm_bindgen]
pub fn export_table(table_json: &str, format: &str, table_name: &str) -> Result<String, JsValue> {
    wasm_export(table_json, format, table_name)
}

#[wasm_bindgen]
pub fn formula_like_cell_count(table_json: &str) -> Result<usize, JsValue> {
    wasm_formula_like_cell_count(table_json)
}

#[wasm_bindgen]
pub fn detect_delimiter(input: &str) -> Result<String, JsValue> {
    wasm_detect_delimiter(input)
}

#[wasm_bindgen]
pub fn get_headers(table_json: &str) -> Result<JsValue, JsValue> {
    wasm_get_headers(table_json)
}
