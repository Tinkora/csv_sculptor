# CSV Sculptor Agent Integration

This document describes the shipped local Agent workflow and tool contract for
the browser-native CSV/TSV workbench. The repository ships `csv_sculptor_mcp`,
a local stdio MCP server. It does not provide a hosted endpoint, persistence,
or authentication.

## Human Workflow

1. Open the browser workbench and import a local UTF-8 CSV/TSV file or paste text.
2. Review the detected delimiter, headers, row count, and parsing result.
3. Filter, sort, hide columns, or deduplicate within the workbench.
4. Review generated JSON, YAML, Markdown, SQL, CSV, or TSV before copying or downloading it.

The browser application keeps input in the current tab. It does not upload
input, but ordinary dependency installation and local HTTP serving remain
outside this data-processing boundary.

## Tool Schemas

`mcp-tools.json` records the five registered tool names and their structured
input shapes. The implementation preserves Rust core behavior, uses versioned
output envelopes, and enforces bounded inputs.

Candidate operations are:

- `csv_sculptor_parse` for bounded CSV/TSV parsing;
- `csv_sculptor_filter` for AND-combined filter conditions;
- `csv_sculptor_sort` for one-column sorting;
- `csv_sculptor_export` for deterministic text output;
- `csv_sculptor_detect_delimiter` for delimiter inspection.

Every successful result has this top-level shape:

```json
{
  "schema_version": "1",
  "tool": "csv_sculptor_parse",
  "data": {}
}
```

Transform and export calls pass the `table` object from the previous result;
they do not accept serialized JSON strings.

## Safety Rules

- Never invent or silently transform user data.
- Treat formula-like spreadsheet values and generated SQL as untrusted text.
- Do not claim that CSV/TSV output is safe to execute or open automatically.
- Do not send input to a network service. The shipped transport is local stdio
  only; any future network transport would require a separate explicit design
  and user authorization.
- Keep the 10 MiB UTF-8 input boundary aligned with the Rust core.
- Keep the stdio JSON line below 64 MiB; oversized lines are discarded before
  dispatch.
- Keep diagnostics on stderr so stdout remains valid MCP JSON-RPC.
