# Contributing to csv_sculptor

[简体中文](CONTRIBUTING.zh-CN.md)

Thanks for your interest in csv_sculptor.

## Development Environment

- Rust 1.95+ (stable)
- wasm-pack 0.15+
- wasm32-unknown-unknown target (`rustup target add wasm32-unknown-unknown`)

## Project Structure

```text
csv_sculptor/
├── crates/
│   ├── csv_sculptor_core/       # Parsing, transform, export logic
│   └── csv_sculptor_web/        # WASM bridge + HTML editor
├── docs/                         # Product spec
├── skills/                       # Agent Skill definitions
└── index.html                    # Landing page
```

## Local Development

```bash
# Run tests
cargo test --workspace

# Format & lint
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

# Build Web WASM
wasm-pack build --target web crates/csv_sculptor_web

# Start local editor
cp crates/csv_sculptor_web/pkg/* crates/csv_sculptor_web/static/pkg/
cd crates/csv_sculptor_web/static && python3 -m http.server 8080
```

## Commit Convention

- Prefix: `feat:` / `fix:` / `docs:` / `refactor:` / `test:` / `chore:`
- Each commit should contain one logically complete change

## Pull Request Process

1. Fork the repository.
2. Create a focused branch whose owned name uses underscores where Git permits.
3. Write an outcome-focused failing test before changing behavior.
4. Run the Rust, browser, documentation, and supply-chain checks affected by the change.
5. Use English Conventional Commit messages in this repository.
6. Open a pull request that states the user-visible outcome and exact verification commands.

Frontend changes must use the `ui-ux-pro-max` design workflow recorded in
`AGENTS.md` and must be checked at 375, 768, 1024, and 1440 pixel widths.

## Code of Conduct

Please read [CODE_OF_CONDUCT.md](./CODE_OF_CONDUCT.md).
