# Contributing to SpecSync

Thank you for your interest in contributing to SpecSync! This guide will help you get started.

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) 1.89 or newer (the CI toolchain is pinned; see `rust-toolchain.toml`)
- Git

### Development Setup

```bash
# Clone the repo
git clone https://github.com/CorvidLabs/spec-sync.git
cd spec-sync

# Build
cargo build

# Run tests
cargo test

# Run lints
cargo clippy -- -D warnings

# Format code
cargo fmt
```

### Before every push (required, keep it fast)

```bash
fledge lanes run pre-push
# or: ./scripts/pre-push-gate.sh
```

Runs **fmt + cargo check + strict path/spec coverage** only (target: ~seconds–2 min warm).  
Does **not** run full `cargo test` or clippy — those are `fledge lanes run verify` / CI. Do not push red.

### Running Locally

```bash
# Validate specs in the current directory
cargo run -- check

# Generate specs for uncovered modules
cargo run -- generate --uncovered

# Run strict validation and bypass the incremental cache
cargo run -- check --strict --force
```

## How to Contribute

### Reporting Bugs

Use the [Bug Report](https://github.com/CorvidLabs/spec-sync/issues/new?template=bug_report.md) issue template. Include:

- SpecSync version (`specsync --version`)
- OS and Rust version
- Minimal reproduction steps
- Expected vs actual behavior

### Suggesting Features

Use the [Feature Request](https://github.com/CorvidLabs/spec-sync/issues/new?template=feature_request.md) issue template. Describe:

- The problem you're trying to solve
- Your proposed solution
- Alternatives you've considered

### Documentation

The documentation marketing site lives in `site/` (Astro + MDX). To work on docs locally:

```bash
cd site
bun install
bun run dev       # dev server at localhost:4321
bun run build     # production build → site/dist/
bun test          # run all site tests
```

Docs content lives in `site/src/content/docs/`. The site is deployed automatically to GitHub Pages on every push to `main` via `.github/workflows/pages.yml`.

### Pull Requests

1. Fork the repo and create a branch from `main`
2. Make your changes
3. Add or update tests as needed
4. Run `cargo test` and `cargo clippy` — everything must pass
5. Update documentation if you changed behavior (`site/src/content/docs/`)
6. Open a PR using the [PR template](.github/PULL_REQUEST_TEMPLATE.md)

### Adding a Language Parser

SpecSync supports 33 languages via extractors in `src/exports/`. To add a new one:

1. Create `src/exports/<language>.rs` implementing an `extract_exports(content: &str) -> Vec<String>` function
2. Register the extractor in `src/exports/mod.rs` and update the `Language` enum in `src/types.rs`
3. Add test fixtures in `tests/fixtures/<language>/`
4. Add tests covering:
   - Export detection (functions, classes, types, constants)
   - Visibility filtering (skip private/internal items)
   - Test file exclusion patterns
5. Update `README.md` with the language in the supported languages table
6. Update `site/src/content/docs/spec-format.md` if the language has any special behaviors

### Commit Messages

Write clear, concise commit messages. Use the imperative mood:

- `fix: handle wildcard re-exports in TypeScript parser`
- `feat: add Elixir language support`
- `docs: update CLI reference for new --format flag`
- `test: add cross-project reference validation tests`

## Code Style

- Follow standard Rust conventions (`cargo fmt`)
- No warnings from `cargo clippy`
- Public items should have doc comments
- Tests go in the same file (`#[cfg(test)]` module) or in `tests/`

## Project Structure

```
src/
  commands/     # CLI command implementations
  exports/      # Language export extractors and AST backends
  parser.rs     # Spec markdown/frontmatter parsing
  validator.rs  # Bidirectional validation and coverage
  generator.rs  # Spec and companion generation
  config.rs     # Configuration loading and migration serialization
tests/
  integration/  # CLI and MCP integration-test modules
specs/          # Module contracts and companion files
site/           # Documentation marketing site (Astro + MDX)
vscode-extension/ # VS Code integration
```

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
