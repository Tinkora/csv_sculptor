# CSV Sculptor Product Specification

[简体中文](product_spec.zh-CN.md)

## Problem and Users

Developers and data workers often need to inspect an unfamiliar CSV/TSV file,
filter a few rows, and convert it for an API, document, or database review.
Desktop spreadsheets are heavy for this job, while online converters require
uploading potentially sensitive data. CSV Sculptor provides an account-free
workbench that runs locally in the browser.

## Evidence-Based Jobs

- Inspect CSV/TSV from build logs, agent traces, test results, and bulk exports.
- Filter, sort, and convert data before placing it in an issue, README,
  configuration file, or script.
- Keep private input away from third-party services.
- Make encoding, size, spreadsheet-formula, and SQL-dialect boundaries explicit.

## Current User Flow

1. Drop a UTF-8 file, choose a file, paste text, or load the sample.
2. Detect comma, tab, pipe, or semicolon and apply the selected header-row mode.
3. Show row and column counts, delimiter, column filters, sortable headers,
   column selection, and a maximum-row control.
4. Combine filters, sort, select columns, limit rows, deduplicate, or reset to
   the imported data.
5. Review and copy or download JSON, YAML, Markdown, SQL, CSV, or TSV.

## Behavioral Contract

- Input is valid UTF-8 and no larger than 10 MiB.
- Header mode rejects blank or duplicate header names.
- Rows with inconsistent field counts are rejected.
- Active filters use AND semantics.
- `GreaterThan` and `LessThan` compare numerically when both operands parse as
  numbers, otherwise they compare text.
- A column sorts numerically only when every value is a finite number;
  otherwise it sorts case-insensitive text.
- The preview shows at most 500 rows while export uses the full current result.
- Explicit column selection and row limits apply to both preview and export.
- Exports are deterministic and preserve field order.

## Security and Privacy

- The application has no upload, analytics, account, persistence, or network API.
- Cells are rendered with DOM `textContent`.
- CSV/TSV preserves formula-like prefixes. The tool must warn about spreadsheet
  formula injection instead of silently changing the data.
- SQL output quotes identifiers and values, but users must still review it for
  the target database dialect and permission model.
- The MCP JSON file is a future integration schema, not an Agent-callable transport.

## Non-Goals

- XLSX, charts, collaborative editing, cloud storage, or share links.
- Streaming files larger than 10 MiB.
- Automatically executing SQL or opening spreadsheet files.
- Regular-expression filters, nested query builders, or persisted projects
  without concrete user evidence.

## Draft Acceptance Gate

- Native Rust formatting, tests, and Clippy pass.
- `wasm32-unknown-unknown` compilation and a real `wasm-pack` build pass.
- Real Chromium exercises import, filter, sort, export, keyboard, and overflow
  behavior at 375, 768, 1024, and 1440 pixel widths.
- English and Chinese documentation are paired and public claims match behavior.
- Dependency audits and GitHub Actions static analysis pass.

Maturity can move from Draft to Alpha only after hosted checks pass for the
exact candidate commit.
