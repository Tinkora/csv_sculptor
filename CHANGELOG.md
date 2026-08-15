# Changelog

All notable changes to this project will be documented in this file. The format
follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the
project intends to use [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0-alpha.3] - 2026-08-15

### Security

- Detect spreadsheet formula-like values after CSV parsing, including tab,
  carriage-return, line-feed, full-width, and optional leading-space variants.
- Keep formula-risk classification in the Rust core while preserving original
  CSV/TSV field values in every export.

## [0.1.0-alpha.2] - 2026-08-14

### Fixed

- Warn before copying or downloading CSV/TSV values that spreadsheet software
  may interpret as formulas, while preserving the original cell values.

## [0.1.0-alpha.1] - 2026-08-14

### Release assets

- Published the browser workbench to GitHub Pages with a reproducible release bundle.
- Added checksums, SPDX SBOM, license inventory, third-party notices, and build attestations.

### Added

- Rust CSV/TSV parsing, filtering, sorting, column selection, deduplication,
  row limiting, and six export formats.
- Browser-local WebAssembly workbench with bilingual controls for column
  selection and row limiting across previews and exports.
- Draft Agent workflow and tool schemas without an MCP transport.
- Bilingual public documentation and repository contract checks.

[0.1.0-alpha.1]: https://github.com/Tinkora/csv_sculptor/releases/tag/v0.1.0-alpha.1
[0.1.0-alpha.2]: https://github.com/Tinkora/csv_sculptor/releases/tag/v0.1.0-alpha.2
[0.1.0-alpha.3]: https://github.com/Tinkora/csv_sculptor/releases/tag/v0.1.0-alpha.3
