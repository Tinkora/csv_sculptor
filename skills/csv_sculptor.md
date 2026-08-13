# CSV Sculptor Agent Integration Draft

This document describes a future Agent workflow and tool contract for the
browser-native CSV/TSV workbench. The repository does not ship an MCP server,
hosted endpoint, authentication, tool registration, or Agent-callable
transport. Do not present the names below as installed tools.

## Human Workflow

1. Open the browser workbench and import a local UTF-8 CSV/TSV file or paste text.
2. Review the detected delimiter, headers, row count, and parsing result.
3. Filter, sort, hide columns, or deduplicate within the workbench.
4. Review generated JSON, YAML, Markdown, SQL, CSV, or TSV before copying or downloading it.

The browser application keeps input in the current tab. It does not upload
input, but ordinary dependency installation and local HTTP serving remain
outside this data-processing boundary.

## Future Tool Schemas

`mcp-tools.json` records candidate names and input shapes for a future
transport. Any implementation must preserve the Rust core behavior and add
versioned output envelopes, bounded inputs, authentication when hosted, and
transport-level tests before this draft can become Agent-callable.

Candidate operations are:

- `csv_sculptor_parse` for bounded CSV/TSV parsing;
- `csv_sculptor_filter` for AND-combined filter conditions;
- `csv_sculptor_sort` for one-column sorting;
- `csv_sculptor_export` for deterministic text output;
- `csv_sculptor_detect_delimiter` for delimiter inspection.

## Safety Rules

- Never invent or silently transform user data.
- Treat formula-like spreadsheet values and generated SQL as untrusted text.
- Do not claim that CSV/TSV output is safe to execute or open automatically.
- Do not send input to a network service unless a future transport documents
  that behavior and the user explicitly authorizes it.
- Keep the 10 MiB UTF-8 input boundary aligned with the Rust core.
