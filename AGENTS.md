# Repository Guide for AI Agents

## Project Overview

csv_sculptor is a browser-native CSV/TSV viewer, filter, sorter, and format converter. Import CSV/TSV data, browse as table, apply filters, sort columns, and export to JSON/YAML/Markdown table/SQL INSERT — all powered by Rust/WASM.

## Architecture

```
csv_sculptor/
├── crates/
│   ├── csv_sculptor_core/       # Parsing, transform, export logic
│   └── csv_sculptor_web/        # WASM bridge + HTML editor
├── docs/                         # Product spec
├── skills/                       # Agent Skill definitions (MCP tools)
└── index.html                    # Product landing page
```

## Key Files for AI Context

| File | Purpose |
|------|---------|
| `crates/csv_sculptor_core/src/parse.rs` | CSV/TSV parsing with auto-detection, CsvTable struct |
| `crates/csv_sculptor_core/src/transform.rs` | Filter, sort, select, deduplicate, limit operations |
| `crates/csv_sculptor_core/src/export.rs` | Export to JSON/YAML/Markdown table/SQL INSERT/CSV/TSV |
| `crates/csv_sculptor_core/src/error.rs` | CoreError enum with stable machine-readable codes |
| `crates/csv_sculptor_core/src/wasm.rs` | WASM bindings for JS interop |
| `crates/csv_sculptor_core/src/lib.rs` | Module declarations and public re-exports |
| `crates/csv_sculptor_web/src/lib.rs` | WASM bridge layer (thin wrapper around core wasm) |
| `crates/csv_sculptor_web/static/index.html` | Full-featured browser editor UI |
| `skills/csv_sculptor.md` | Agent usage workflow |
| `skills/mcp-tools.json` | MCP tool definitions |

## Build & Test Commands

```bash
# Run all tests
cargo test --workspace

# Format check
cargo fmt --all -- --check

# Lint (strict)
cargo clippy --workspace --all-targets -- -D warnings

# WASM compilation check
cargo check -p csv_sculptor_web --target wasm32-unknown-unknown

# Build Web WASM for deployment
wasm-pack build --target web crates/csv_sculptor_web
```

## Design Principles

1. **Browser-first**: All CSV/TSV processing happens in-browser via WASM. No data is uploaded to a server.
2. **Zero-copy when possible**: The WASM bridge serializes CsvTable to JSON for JS interop, enabling stateless function calls.
3. **Auto-detection**: Delimiter (comma, tab, pipe, semicolon) is auto-detected from input content.
4. **Streaming-ready**: The core API is designed to work with owned data, making it compatible with future streaming parsers.
5. **Format-preserving**: Round-trip CSV → CsvTable → CSV preserves data integrity.

## Core Data Model

```
CsvTable {
    headers: Vec<String>,    // Column names
    rows: Vec<Vec<String>>,  // Data rows (each row is a Vec of field values)
    delimiter: char,         // Detected or specified delimiter
    row_count: usize,        // Total number of data rows
}
```

## Filter Operators

| Operator | Description |
|----------|-------------|
| Equals | Exact string match |
| NotEquals | String does not match |
| Contains | String contains substring (case-insensitive) |
| StartsWith | String starts with prefix |
| EndsWith | String ends with suffix |
| GreaterThan | Numeric comparison > |
| LessThan | Numeric comparison < |
| IsEmpty | Field is empty or whitespace-only |
| IsNotEmpty | Field has non-whitespace content |

## Export Formats

| Format | Function | Description |
|--------|----------|-------------|
| JSON | `to_json()` | Array of objects with header keys |
| YAML | `to_yaml()` | YAML array of objects |
| Markdown | `to_markdown_table()` | GitHub-flavored markdown table |
| SQL | `to_sql_insert()` | INSERT INTO ... VALUES statements |
| CSV | `to_csv()` | Comma-separated values |
| TSV | `to_tsv()` | Tab-separated values |

## Error Codes (Stable Machine-Readable)

| Code | Meaning |
|------|---------|
| `PARSE_ERROR` | CSV/TSV parsing failed |
| `DELIMITER_DETECTION_FAILED` | Could not auto-detect delimiter |
| `COLUMN_NOT_FOUND` | Referenced column does not exist in headers |
| `INVALID_FILTER_OPERATOR` | Filter operator not recognized |
| `EMPTY_TABLE` | Operation on empty table |
| `EXPORT_ERROR` | Export format serialization failed |
| `SERIALIZATION_ERROR` | JSON serialization/deserialization failed |
| `INVALID_INPUT` | Input data is malformed or empty |

## Commit Language

- Write commit subjects and bodies in English and follow Conventional Commits.
- This repository-level rule overrides any global preference for another commit-message language.

## Frontend Design Requirement

- Before creating, modifying, reviewing, or debugging any HTML page or user-facing frontend, invoke the `ui-ux-pro-max` skill.
- Run the skill's required `--design-system` search before editing, followed by relevant stack and UX searches.
- If `ui-ux-pro-max` is unavailable, stop frontend work and report the missing prerequisite.
- Verify the rendered result in a real browser at 375, 768, 1024, and 1440 pixel widths, including console, keyboard, accessibility, and overflow checks.
