# Contributing to SpecSync

Thank you for your interest in contributing to SpecSync! This guide will help you get started.

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable, 1.85+)
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

### Running Locally

```bash
# Validate specs in the current directory
cargo run -- validate

# Generate a spec from source files
cargo run -- generate src/parser.rs

# Check with verbose output
cargo run -- validate --verbose
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

### Pull Requests

1. Fork the repo and create a branch from `main`
2. Make your changes
3. Add or update tests as needed
4. Run `cargo test` and `cargo clippy` — everything must pass
5. Update documentation if you changed behavior
6. Open a PR using the [PR template](.github/PULL_REQUEST_TEMPLATE.md)

### Adding a Language Parser

SpecSync supports 11 languages via parsers in `src/parser/`. To add a new one:

1. Create `src/parser/<language>.rs` implementing the `Parser` trait
2. Register the parser in `src/parser/mod.rs` with its file extensions
3. Add test fixtures in `tests/fixtures/<language>/`
4. Add tests covering:
   - Export detection (functions, classes, types, constants)
   - Visibility filtering (skip private/internal items)
   - Test file exclusion patterns
5. Update `README.md` with the language in the supported languages table
6. Update `docs/spec-format.md` if the language has any special behaviors

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
  parser/       # Language parsers (one per language)
  validator/    # Spec validation logic
  generator/    # Spec generation from source
  reporter/     # Output formatting (text, JSON, SARIF)
  config/       # Configuration loading
tests/
  fixtures/     # Test fixture files per language
  integration/  # Integration tests
docs/           # Documentation site (Jekyll)
```

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
