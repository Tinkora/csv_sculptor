# Security Policy

[简体中文](SECURITY.zh-CN.md)

## Supported Versions

| Version | Supported |
|---------|-----------|
| `main` | Yes |
| Unreleased Draft snapshots | No stability guarantee |

## Reporting a Vulnerability

If you discover a security vulnerability, please **do not** open a public issue.

Use GitHub private vulnerability reporting for this repository when available.
If that private channel is unavailable, do not include vulnerability details
in a public issue; contact the Tinkora organization owner through an already
established private channel. No response-time guarantee is made until a
monitored security contact is published.

### Scope

The following areas are within scope:

- WASM sandbox escape vectors
- CSV injection attacks (malformed input causing crashes or OOM)
- Export format injection (spreadsheet formulas, SQL, YAML, Markdown)
- XSS vectors in the HTML editor UI
- Path traversal or file-system access from WASM

### Out of Scope

- Issues already documented as known limitations
- Theoretical attacks requiring physical access
- Issues in dependencies (please report upstream)

## Security Model

The csv_sculptor project follows these security principles:

1. **Browser-local processing**: All CSV/TSV parsing, transform, and export happens in-browser via WASM. No user data touches any server.

2. **No persistent storage**: The editor keeps all data in JavaScript memory; there is no server-side storage, no database, and no worker.

3. **Safe rendering**: User-provided text is assigned with `textContent` before rendering in the HTML table.

4. **WASM memory safety**: Rust's ownership model and the `csv` crate provide memory-safe parsing even for malformed input.

5. **No eval or innerHTML injection**: The editor renders table cells via DOM API (`textContent`) rather than `innerHTML` to prevent script injection.

CSV and TSV exports deliberately preserve input values. They do not neutralize
formula-like prefixes, so opening untrusted output in spreadsheet software is
a separate trust boundary. SQL export produces reviewable text and never
executes it.
