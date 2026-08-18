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
- Support the encoding choices that commonly appear in spreadsheet and Agent
  batch workflows. OpenAI Codex's merged [`spawn_agents_on_csv` workflow](https://github.com/openai/codex/pull/10935)
  treats CSV as a first-class Agent work manifest, while reports such as
  [Directus #12970](https://github.com/directus/directus/issues/12970) show
  that Excel-generated UTF-8 BOM files can break downstream imports.

## Current User Flow

1. Drop or choose a file, select an encoding, paste UTF-8 text, or load the sample.
2. Auto-detect UTF-16 BOMs or explicitly choose UTF-8, UTF-16 LE/BE, or
   Windows-1252 before decoding a browser file.
3. Detect comma, tab, pipe, or semicolon and apply the selected header-row mode.
4. Show row and column counts, delimiter, column filters, sortable headers,
   column selection, and a maximum-row control.
5. Combine filters, sort, select columns, limit rows, deduplicate, or reset to
   the imported data.
6. Review and copy or download JSON, YAML, Markdown, SQL, CSV, or TSV.

## Agent Workflow

1. Build `csv_sculptor_mcp` and register the binary as a local MCP stdio server.
2. Call `csv_sculptor_parse` with bounded UTF-8 text.
3. Pass the returned structured `table` directly to `csv_sculptor_filter`,
   `csv_sculptor_sort`, or `csv_sculptor_export`.
4. Inspect the versioned output envelope and any export warnings before using
   generated text.

The server does not provide hosted transport, authentication, persistence, or
network access. It writes diagnostics to stderr and keeps protocol JSON on
stdout.

## Behavioral Contract

- Pasted text and MCP input are valid UTF-8 and no larger than 10 MiB; browser
  file bytes follow the encoding rule below.
- Browser file imports may use UTF-8, UTF-16 LE/BE, or Windows-1252. Auto mode
  recognizes UTF-16 BOMs and otherwise uses strict UTF-8; malformed bytes are
  rejected rather than replaced. Pasted text and MCP input remain UTF-8.
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
- Agent results use the envelope `{ "schema_version": "1", "tool": "...", "data": ... }`.
- Agent table inputs must have unique non-blank headers, consistent row widths,
  a supported delimiter, and a total cell-data size no larger than 10 MiB.
- MCP stdio JSON lines are capped at 64 MiB; over-limit lines are discarded
  before tool dispatch.

## Security and Privacy

- The application has no upload, analytics, account, persistence, or network API.
- File decoding is performed locally with an explicit supported encoding; the
  application does not guess arbitrary legacy encodings or silently repair
  invalid bytes.
- Cells are rendered with DOM `textContent`.
- Formula detection runs on parsed fields, not raw lines. Direct prefixes cover
  `=`, `+`, `-`, `@`, tab, carriage return, line feed, and their full-width
  variants as documented by [OWASP CSV Injection](https://owasp.org/www-community/attacks/CSV_Injection).
- Detection also checks after optional leading ASCII spaces because LibreOffice
  can remove them with its [Trim spaces import option](https://help.libreoffice.org/latest/en-US/text/shared/00/00000208.html).
  The scanner does not remove those spaces or change the field value; RFC 4180
  treats spaces as field content.
- CSV/TSV preserves formula-like prefixes and warns instead of silently changing
  the data. [CWE-1236](https://cwe.mitre.org/data/definitions/1236.html) notes that
  mitigations vary across spreadsheet products, so the warning is not a claim of
  universal sanitization.
- SQL output quotes identifiers and values, but users must still review it for
  the target database dialect and permission model.
- The local stdio MCP server is Agent-callable; `skills/mcp-tools.json` documents
  its five registered tools. No hosted endpoint or authentication is provided.

## Non-Goals

- XLSX, charts, collaborative editing, cloud storage, or share links.
- Streaming files larger than 10 MiB or a hosted MCP service.
- Automatically executing SQL or opening spreadsheet files.
- Regular-expression filters, nested query builders, or persisted projects
  without concrete user evidence.

## Alpha Acceptance Gate

- Native Rust formatting, tests, and Clippy pass.
- `wasm32-unknown-unknown` compilation and a real `wasm-pack` build pass.
- Real Chromium exercises import, filter, sort, export, keyboard, and overflow
  behavior at 375, 768, 1024, and 1440 pixel widths.
- English and Chinese documentation are paired and public claims match behavior.
- Dependency audits and GitHub Actions static analysis pass.

Alpha releases require the hosted quality, supply-chain, documentation, and
browser checks to pass for the exact candidate commit. MCP behavior additionally
requires the local tool and bounded-stdio tests to pass.
