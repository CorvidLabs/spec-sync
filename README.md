<div align="center">

# SpecSync

[![GitHub Marketplace](https://img.shields.io/badge/Marketplace-SpecSync-blue?logo=github)](https://github.com/marketplace/actions/spec-sync)
[![CI](https://github.com/CorvidLabs/spec-sync/actions/workflows/ci.yml/badge.svg)](https://github.com/CorvidLabs/spec-sync/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/specsync.svg)](https://crates.io/crates/specsync)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

**Bidirectional spec-to-code validation.** Written in Rust. Single binary. 9 languages.

[Quick Start](#quick-start) &bull; [Spec Format](#spec-format) &bull; [CLI](#cli-reference) &bull; [GitHub Action](#github-action) &bull; [Config](#configuration) &bull; [Docs Site](https://corvidlabs.github.io/spec-sync)

</div>

---

## What It Does

SpecSync validates markdown module specs (`*.spec.md`) against your source code in both directions:

| Direction | Severity | Meaning |
|-----------|----------|---------|
| Code exports something not in the spec | Warning | Undocumented export |
| Spec documents something missing from code | **Error** | Stale/phantom entry |
| Source file in spec was deleted | **Error** | Missing file |
| DB table in spec missing from schema | **Error** | Phantom table |
| Required markdown section missing | **Error** | Incomplete spec |

## Supported Languages

Auto-detected from file extensions. Same spec format for all.

| Language | Exports Detected | Test Exclusions |
|----------|-----------------|-----------------|
| **TypeScript/JS** | `export function/class/type/const/enum`, re-exports | `.test.ts`, `.spec.ts`, `.d.ts` |
| **Rust** | `pub fn/struct/enum/trait/type/const/static/mod` | `#[cfg(test)]` modules |
| **Go** | Uppercase `func/type/var/const`, methods | `_test.go` |
| **Python** | `__all__`, or top-level `def/class` (no `_` prefix) | `test_*.py`, `*_test.py` |
| **Swift** | `public/open` func/class/struct/enum/protocol/actor | `*Tests.swift` |
| **Kotlin** | Top-level declarations (excludes private/internal) | `*Test.kt`, `*Spec.kt` |
| **Java** | `public` class/interface/enum/record/methods | `*Test.java`, `*Tests.java` |
| **C#** | `public` class/struct/interface/enum/record/delegate | `*Test.cs`, `*Tests.cs` |
| **Dart** | Top-level (no `_` prefix), class/mixin/enum/typedef | `*_test.dart` |

---

## Install

### GitHub Action (recommended)

```yaml
- uses: CorvidLabs/spec-sync@v1
  with:
    strict: 'true'
    require-coverage: '100'
```

### Crates.io

```bash
cargo install specsync
```

### Pre-built binaries

Download from [GitHub Releases](https://github.com/CorvidLabs/spec-sync/releases):

```bash
# macOS (Apple Silicon)
curl -sL https://github.com/CorvidLabs/spec-sync/releases/latest/download/specsync-macos-aarch64.tar.gz | tar xz
sudo mv specsync-macos-aarch64 /usr/local/bin/specsync

# macOS (Intel)
curl -sL https://github.com/CorvidLabs/spec-sync/releases/latest/download/specsync-macos-x86_64.tar.gz | tar xz
sudo mv specsync-macos-x86_64 /usr/local/bin/specsync

# Linux (x86_64)
curl -sL https://github.com/CorvidLabs/spec-sync/releases/latest/download/specsync-linux-x86_64.tar.gz | tar xz
sudo mv specsync-linux-x86_64 /usr/local/bin/specsync

# Linux (aarch64)
curl -sL https://github.com/CorvidLabs/spec-sync/releases/latest/download/specsync-linux-aarch64.tar.gz | tar xz
sudo mv specsync-linux-aarch64 /usr/local/bin/specsync
```

**Windows:** download `specsync-windows-x86_64.exe.zip` from the [releases page](https://github.com/CorvidLabs/spec-sync/releases).

### From source

```bash
cargo install --git https://github.com/CorvidLabs/spec-sync
```

---

## Quick Start

```bash
specsync init                              # Create specsync.json config
specsync check                             # Validate specs against code
specsync coverage                          # Show file/module coverage
specsync generate                          # Scaffold specs for unspecced modules
specsync watch                             # Re-validate on every file change
```

---

## Spec Format

Specs are markdown files (`*.spec.md`) with YAML frontmatter in your specs directory.

### Frontmatter

```yaml
---
module: auth                                # Module name (required)
version: 3                                  # Spec version (required)
status: stable                              # draft | review | stable | deprecated (required)
files:                                      # Source files covered (required, non-empty)
  - src/auth/service.ts
  - src/auth/middleware.ts
db_tables:                                  # Validated against schema dir (optional)
  - users
  - sessions
depends_on:                                 # Other spec paths, validated for existence (optional)
  - specs/database/database.spec.md
---
```

### Required Sections

Every spec must include these `##` sections (configurable in `specsync.json`):

Purpose, Public API, Invariants, Behavioral Examples, Error Cases, Dependencies, Change Log

### Public API Tables

SpecSync extracts the first backtick-quoted name per row and cross-references it against code exports:

```markdown
## Public API

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `authenticate` | `(token: string)` | `User \| null` | Validates bearer token |
| `refreshSession` | `(sessionId: string)` | `Session` | Extends session TTL |
```

<details>
<summary>Full spec example</summary>

```markdown
---
module: auth
version: 3
status: stable
files:
  - src/auth/service.ts
  - src/auth/middleware.ts
db_tables:
  - users
  - sessions
depends_on:
  - specs/database/database.spec.md
---

# Auth

## Purpose

Handles authentication and session management. Validates bearer tokens,
manages session lifecycle, provides middleware for route protection.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `authenticate` | `(token: string)` | `User \| null` | Validates a token |
| `refreshSession` | `(sessionId: string)` | `Session` | Extends session TTL |

### Exported Types

| Type | Description |
|------|-------------|
| `User` | Authenticated user object |
| `Session` | Active session record |

## Invariants

1. Sessions expire after 24 hours
2. Failed auth attempts rate-limited to 5/minute
3. Tokens validated cryptographically, never by string comparison

## Behavioral Examples

### Scenario: Valid token

- **Given** a valid JWT token
- **When** `authenticate()` is called
- **Then** returns the corresponding User object

### Scenario: Expired token

- **Given** an expired JWT token
- **When** `authenticate()` is called
- **Then** returns null and logs a warning

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Expired token | Returns null, logs warning |
| Malformed token | Returns null |
| DB unavailable | Throws `ServiceUnavailableError` |

## Dependencies

| Module | Usage |
|--------|-------|
| database | `query()` for user lookups |
| crypto | `verifyJwt()` for token validation |

## Change Log

| Date | Change |
|------|--------|
| 2026-03-18 | Initial spec |
```

</details>

---

## CLI Reference

```
specsync [command] [flags]
```

### Commands

| Command | Description |
|---------|-------------|
| `check` | Validate all specs against source code **(default)** |
| `coverage` | File and module coverage report |
| `generate` | Scaffold specs for modules missing one |
| `init` | Create default `specsync.json` |
| `watch` | Live validation on file changes (500ms debounce) |

### Flags

| Flag | Description |
|------|-------------|
| `--strict` | Warnings become errors (recommended for CI) |
| `--require-coverage N` | Fail if file coverage < N% |
| `--root <path>` | Project root (default: cwd) |
| `--json` | Structured JSON output |

### Exit Codes

| Code | Meaning |
|------|---------|
| `0` | All checks passed |
| `1` | Errors, strict warnings, or coverage below threshold |

---

## GitHub Action

Available on the [GitHub Marketplace](https://github.com/marketplace/actions/spec-sync). Auto-detects OS/arch, downloads the binary, runs validation.

### Inputs

| Input | Default | Description |
|-------|---------|-------------|
| `version` | `latest` | Release version to download |
| `strict` | `false` | Treat warnings as errors |
| `require-coverage` | `0` | Minimum file coverage % |
| `root` | `.` | Project root directory |
| `args` | `''` | Extra CLI arguments |

### Workflow example

```yaml
name: Spec Check
on: [push, pull_request]

jobs:
  specsync:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: CorvidLabs/spec-sync@v1
        with:
          strict: 'true'
          require-coverage: '100'
```

---

## Configuration

Create `specsync.json` in your project root (or run `specsync init`):

```json
{
  "specsDir": "specs",
  "sourceDirs": ["src"],
  "schemaDir": "db/migrations",
  "requiredSections": ["Purpose", "Public API", "Invariants", "Behavioral Examples", "Error Cases", "Dependencies", "Change Log"],
  "excludeDirs": ["__tests__"],
  "excludePatterns": ["**/__tests__/**", "**/*.test.ts", "**/*.spec.ts"],
  "sourceExtensions": []
}
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `specsDir` | `string` | `"specs"` | Directory containing `*.spec.md` files |
| `sourceDirs` | `string[]` | `["src"]` | Source directories for coverage analysis |
| `schemaDir` | `string?` | — | SQL schema dir for `db_tables` validation |
| `schemaPattern` | `string?` | `CREATE TABLE` regex | Custom regex for table name extraction |
| `requiredSections` | `string[]` | 7 defaults | Markdown sections every spec must include |
| `excludeDirs` | `string[]` | `["__tests__"]` | Directories excluded from coverage |
| `excludePatterns` | `string[]` | Common test globs | File patterns excluded from coverage |
| `sourceExtensions` | `string[]` | All supported | Restrict to specific extensions (e.g., `["ts", "rs"]`) |

---

## Spec Generation

`specsync generate` scans your source directories, finds modules without spec files, and scaffolds `*.spec.md` files for each one — frontmatter populated, source files listed, all required sections stubbed.

```bash
specsync generate                         # Scaffold specs for all unspecced modules
specsync coverage                         # See what's still missing
```

### How it works

1. Runs coverage analysis to find modules with no corresponding spec
2. For each unspecced module, discovers all source files (excluding tests)
3. Generates a spec using your custom template (`specs/_template.spec.md`) or the built-in default
4. Writes `specs/<module>/<module>.spec.md` with `module`, `files`, `version: 1`, `status: draft` pre-filled

### Custom templates

Drop a `_template.spec.md` in your specs directory. The generator replaces `module`, `version`, `status`, `files`, and the `# Title` heading — everything else stays as-is. Use this to enforce your team's spec structure.

### Designed for AI agents

The generate command is the entry point for LLM-powered spec workflows. A coding agent can:

```bash
specsync generate                                  # scaffold specs for new modules
# LLM fills in Purpose, Public API, Invariants...  # agent writes the content
specsync check --json                              # validate, get structured feedback
# LLM fixes errors from JSON output                # iterate until clean
specsync check --strict --require-coverage 100     # enforce full coverage in CI
```

Every output format is designed for machine consumption:
- **`--json`** on any command → structured JSON, no ANSI codes
- **Exit code 0/1** → pass/fail, no parsing needed
- **Spec files are plain markdown** → any LLM can read and write them
- **Public API tables** use backtick-quoted names → unambiguous to extract

### JSON output shapes

```json
// specsync check --json
{ "passed": false, "errors": ["..."], "warnings": ["..."], "specs_checked": 12 }

// specsync coverage --json
{ "file_coverage": 85.33, "files_covered": 23, "files_total": 27, "modules": [{"name": "helpers", "has_spec": false}] }
```

---

## Architecture

```
src/
├── main.rs            CLI entry + output formatting
├── types.rs           Data types + config schema
├── config.rs          specsync.json loading
├── parser.rs          Frontmatter + spec body parsing
├── validator.rs       Validation + coverage computation
├── generator.rs       Spec scaffolding
├── watch.rs           File watcher (notify, 500ms debounce)
└── exports/
    ├── mod.rs          Language dispatch
    ├── typescript.rs   TS/JS exports
    ├── rust_lang.rs    Rust pub items
    ├── go.rs           Go uppercase identifiers
    ├── python.rs       Python __all__ / top-level
    ├── swift.rs        Swift public/open items
    ├── kotlin.rs       Kotlin top-level
    ├── java.rs         Java public items
    ├── csharp.rs       C# public items
    └── dart.rs         Dart public items
```

**Design:** Single binary, no runtime deps. Frontmatter parsed with regex (no YAML library). Language backends use regex, not ASTs — works without compilers installed. Release builds use LTO + strip + opt-level 3.

---

## Contributing

1. Fork, branch (`git checkout -b feat/my-feature`), implement
2. `cargo test` + `cargo clippy`
3. Open a PR

### Adding a language

1. Create `src/exports/yourlang.rs` — return `Vec<String>` of exported names
2. Add variant to `Language` enum in `types.rs`
3. Wire extension detection + dispatch in `src/exports/mod.rs`
4. Add tests for common patterns

---

## License

[MIT](LICENSE) &copy; [CorvidLabs](https://github.com/CorvidLabs)
