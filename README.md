# CSV Sculptor

CSV Sculptor is a browser-native CSV/TSV workbench for inspecting, filtering,
sorting, and converting tabular text without uploading the input. Rust owns the
data behavior and a thin WebAssembly boundary exposes it to the browser.

[简体中文](README.zh-CN.md)

## Maturity

**Alpha.** The hosted quality and release workflows pass for the published
candidate, the browser workbench is deployed to GitHub Pages, and the release
bundle includes checksums, an SPDX SBOM, a license inventory, and attestations.

- **Try it:** [GitHub Pages](https://tinkora.github.io/csv_sculptor/)
- **Latest candidate:** [v0.1.0-alpha.4 release](https://github.com/Tinkora/csv_sculptor/releases/tag/v0.1.0-alpha.4)

- **Local human interface:** implemented and covered by hosted Chromium smoke tests.
- **Local Agent interface:** `csv_sculptor_mcp` exposes five MCP tools over stdio.
- **No hosted service:** the MCP server has no network transport, account, or
  authentication requirement; input stays in the local process.

## Current Scope

- Parse UTF-8 CSV, TSV, pipe-delimited, and semicolon-delimited text up to
  10 MiB, including quoted fields.
- Reject blank or duplicate headers and inconsistent row widths.
- Filter with nine operators and combine active filters with AND semantics.
- Sort numeric columns numerically and other columns case-insensitively.
- Select columns, limit rows, and remove duplicate rows from the current result.
- Export JSON, YAML, Markdown tables, SQL `INSERT`, CSV, or TSV.
- Keep all imported data in the current browser tab.
- Switch the workbench between English and Simplified Chinese.

## Agent Integration

Build the local MCP server from the repository root:

```bash
cargo build --release -p csv_sculptor_mcp --locked
```

Register the resulting `target/release/csv_sculptor_mcp` binary in an MCP
client's local stdio configuration. The server exposes:

| Tool | Purpose |
| --- | --- |
| `csv_sculptor_parse` | Parse bounded CSV/TSV text and return a structured table |
| `csv_sculptor_filter` | Apply AND-combined filter conditions |
| `csv_sculptor_sort` | Sort one table column numerically or as text |
| `csv_sculptor_export` | Produce deterministic JSON, YAML, Markdown, SQL, CSV, or TSV text |
| `csv_sculptor_detect_delimiter` | Inspect the delimiter without parsing a table |

Transform and export tools accept a structured `table` object, so an Agent can
chain calls without embedding JSON strings inside JSON. Every successful result
uses the envelope `{ "schema_version": "1", "tool": "...", "data": ... }`.
Invalid input is returned as an MCP tool error with a stable core error code.

The raw CSV/TSV data limit is 10 MiB. The stdio JSON line limit is 64 MiB to
allow escaped representations, and oversized lines are discarded before tool
dispatch. The server writes diagnostics to stderr and protocol messages only to
stdout. See [`skills/mcp-tools.json`](skills/mcp-tools.json) for the machine-
readable catalog.

Column selection and row limiting apply to both the browser preview and every
export format. Reset restores the complete imported table.

## Safety Boundaries

- Input is decoded as UTF-8 and never sent by the application to a server.
- The preview renders cell values with `textContent`, not HTML.
- CSV/TSV exports preserve cell values. The export dialog warns when a parsed
  field begins, after optional ASCII spaces, with an ASCII or full-width formula
  prefix identified by the documented security policy. It does not rewrite the
  data; review untrusted exports before opening them in spreadsheet software.
- SQL output quotes identifiers and string values, but it is generated text,
  not a database migration. Review it against the target database dialect.
- The browser preview is capped at 500 rows; transformations and exports still
  operate on the full in-memory table.

## Develop

Requirements:

- Rust 1.95.0 with the `wasm32-unknown-unknown` target
- `wasm-pack` 0.15.0
- Node.js 24 or newer for browser smoke tests

Run the Rust checks from the repository root:

```bash
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo check -p csv_sculptor_web --target wasm32-unknown-unknown --locked
```

Build the real WebAssembly package and run the workbench:

```bash
cd crates/csv_sculptor_web
wasm-pack build --target web --out-dir static/pkg .
npm ci --ignore-scripts
npm run serve
```

Open `http://127.0.0.1:4173/static/`.

Run the browser smoke suite at 375, 768, 1024, and 1440 pixel widths:

```bash
cd crates/csv_sculptor_web
npm run test:wasm-smoke
```

Run documentation and supply-chain checks:

```bash
ruby scripts/test_check_docs.rb
ruby scripts/check_docs.rb
cargo deny check advisories bans licenses sources
cargo audit --no-yanked
```

`cargo deny` owns the yanked-package gate; `cargo audit --no-yanked` performs the
independent RustSec advisory scan without duplicating registry API requests.

Generated `target/`, `pkg/`, `node_modules/`, Playwright results, and browser
artifacts are ignored and must not be committed.

## Structure

| Path | Responsibility |
| --- | --- |
| `crates/csv_sculptor_core` | Parsing, transformations, exports, and stable errors |
| `crates/csv_sculptor_web` | Thin WASM boundary and browser workbench |
| `crates/csv_sculptor_mcp` | Local stdio MCP server and bounded Agent tools |
| `skills/` | Agent-facing workflow and machine-readable tool schemas |
| `docs/` | Bilingual product contract |
| `scripts/` | Offline repository contract checks |

## Contributing and Support

- [Contributing](CONTRIBUTING.md)
- [Security policy](SECURITY.md)
- [Support](SUPPORT.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)
- [Changelog](CHANGELOG.md)

[Support Tinkora on Ko-fi](https://ko-fi.com/tinkora)

## License

MIT. See [LICENSE](LICENSE).
