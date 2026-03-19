<div align="center">

# SpecSync

[![GitHub Marketplace](https://img.shields.io/badge/Marketplace-SpecSync-blue?logo=github)](https://github.com/marketplace/actions/spec-sync)

**Bidirectional spec-to-code validation — keep your docs honest.**

Written in Rust. Language-agnostic. Blazing fast.

[![CI](https://github.com/CorvidLabs/spec-sync/actions/workflows/ci.yml/badge.svg)](https://github.com/CorvidLabs/spec-sync/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/specsync.svg)](https://crates.io/crates/specsync)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

[Quick Start](#quick-start) &bull; [Spec Format](#spec-format) &bull; [CLI Reference](#cli-reference) &bull; [GitHub Action](#github-action) &bull; [Configuration](#configuration) &bull; [Documentation Site](https://corvidlabs.github.io/spec-sync)

</div>

---

## The Problem

Documentation drifts. Engineers add new exports but forget to update the spec. Specs reference functions that were renamed months ago. Nobody notices until a new team member reads the docs and gets confused.

**SpecSync catches this automatically.**

## What It Does

SpecSync validates your markdown module specs against actual source code — in both directions:

| Situation | Severity | Message |
|-----------|----------|---------|
| Code exports something not in the spec | Warning | Undocumented export |
| Spec documents something that doesn't exist | **Error** | Stale/phantom spec entry |
| Source file referenced in spec was deleted | **Error** | Missing file |
| DB table in spec doesn't exist in schema | **Error** | Phantom table |
| Required section missing from spec | **Error** | Incomplete spec |

## Supported Languages

SpecSync auto-detects the language from file extensions. The same spec format works for all of them.

| Language | What Gets Detected | Test Files Excluded |
|----------|--------------------|---------------------|
| **TypeScript / JavaScript** | `export function`, `export class`, `export type`, `export const`, `export enum`, re-exports | `.test.ts`, `.spec.ts`, `.d.ts` |
| **Rust** | `pub fn`, `pub struct`, `pub enum`, `pub trait`, `pub type`, `pub const`, `pub static`, `pub mod` | Inline `#[cfg(test)]` modules |
| **Go** | Uppercase identifiers: `func`, `type`, `var`, `const`, methods | `_test.go` |
| **Python** | `__all__` list, or top-level `def`/`class` (excluding `_`-prefixed) | `test_*.py`, `*_test.py` |
| **Swift** | `public`/`open` func, class, struct, enum, protocol, typealias, actor, init | `*Tests.swift`, `*Test.swift` |
| **Kotlin** | Top-level declarations (public by default), excludes `private`/`internal`/`protected` | `*Test.kt`, `*Spec.kt` |
| **Java** | `public` class, interface, enum, record, methods, fields | `*Test.java`, `*Tests.java` |
| **C#** | `public` class, struct, interface, enum, record, delegate, methods | `*Test.cs`, `*Tests.cs` |
| **Dart** | Top-level declarations (no `_` prefix), class, mixin, enum, typedef | `*_test.dart` |

---

## Install

### GitHub Action (recommended)

Available on the [GitHub Marketplace](https://github.com/marketplace/actions/spec-sync). Add to any workflow:

```yaml
- uses: CorvidLabs/spec-sync@v1
  with:
    strict: 'true'
    require-coverage: '100'
```

No binary download or Rust toolchain needed — the action handles everything.

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

# Or clone and build
git clone https://github.com/CorvidLabs/spec-sync.git
cd spec-sync && cargo install --path .
```

### Crates.io

```bash
cargo install specsync
```

---

## Quick Start

```bash
# 1. Initialize config in your project root
specsync init

# 2. Validate all specs
specsync check

# 3. See what's covered and what's missing
specsync coverage

# 4. Auto-generate specs for unspecced modules
specsync generate

# 5. Watch mode — re-validates on every file change
specsync watch
```

---

## Spec Format

Specs are markdown files (`*.spec.md`) with YAML frontmatter. Place them in your specs directory (default: `specs/`).

### Frontmatter

```yaml
---
module: auth              # Module name (required)
version: 3                # Spec version (required)
status: stable            # draft | review | stable | deprecated (required)
files:                    # Source files this spec covers (required, non-empty)
  - src/auth/service.ts
  - src/auth/middleware.ts
db_tables:                # DB tables used (optional, validated against schema)
  - users
  - sessions
depends_on:               # Other specs this module depends on (optional)
  - specs/database/database.spec.md
---
```

### Required Sections

By default, every spec must include these markdown sections (configurable):

| Section | Purpose |
|---------|---------|
| `## Purpose` | What this module does and why it exists |
| `## Public API` | Tables listing exported symbols — this is what gets validated against code |
| `## Invariants` | Rules that must always hold true |
| `## Behavioral Examples` | Given/When/Then scenarios |
| `## Error Cases` | How the module handles failures |
| `## Dependencies` | What this module consumes from other modules |
| `## Change Log` | History of spec changes |

### Public API Tables

The Public API section uses markdown tables with backtick-quoted symbol names. SpecSync extracts these and cross-references them against actual code exports.

```markdown
## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `authenticate` | `(token: string)` | `User \| null` | Validates a bearer token |
| `refreshSession` | `(sessionId: string)` | `Session` | Extends session TTL |

### Exported Types

| Type | Description |
|------|-------------|
| `User` | Authenticated user object |
| `Session` | Active session record |
```

### Full Example

<details>
<summary>Click to expand a complete spec file</summary>

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
manages session lifecycle, and provides middleware for route protection.

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
2. Failed auth attempts are rate-limited to 5/minute
3. Tokens are validated cryptographically, never by string comparison

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

### Consumes

| Module | What is used |
|--------|-------------|
| database | `query()` for user lookups |
| crypto | `verifyJwt()` for token validation |

## Change Log

| Date | Author | Change |
|------|--------|--------|
| 2026-03-18 | team | Initial spec |
```

</details>

---

## CLI Reference

```
specsync [command] [flags]
<<<<<<< HEAD

Commands:
  check       Validate all specs against source (default)
  coverage    Show file and module coverage report
  generate    Scaffold specs for unspecced modules
  init        Create specsync.json config file
  watch       Watch for file changes and re-validate
  help        Show help

Flags:
  --strict              Treat warnings as errors
  --require-coverage N  Fail if file coverage < N%
  --root <path>         Project root (default: cwd)
  --json                Output results as JSON
=======
>>>>>>> fed5ec7 (docs: add GitHub Pages documentation site)
```

### Commands

| Command | Description |
|---------|-------------|
| `check` | Validate all specs against source code **(default)** |
| `coverage` | Show file and module coverage report |
| `generate` | Scaffold spec files for modules that don't have one |
| `init` | Create a default `specsync.json` config file |
| `watch` | Live validation — re-runs on file changes (500ms debounce) |

### Flags

| Flag | Description |
|------|-------------|
| `--strict` | Treat warnings as errors (recommended for CI) |
| `--require-coverage N` | Fail if file coverage is below N% |
| `--root <path>` | Set project root directory (default: current directory) |
| `--json` | Output structured JSON instead of colored text |

### Examples

```bash
# Basic validation
specsync check

# Strict mode for CI — warnings fail the build
specsync check --strict

# Enforce 100% spec coverage
specsync check --strict --require-coverage 100

# JSON output for tooling integration
specsync check --json

# Override project root
specsync check --root ./packages/backend

# Watch mode for development
specsync watch
```

### Exit Codes

| Code | Meaning |
|------|---------|
| `0` | All checks passed |
| `1` | Errors found, or warnings found with `--strict`, or coverage below threshold |

---

## GitHub Action

<<<<<<< HEAD
Available on the [GitHub Marketplace](https://github.com/marketplace/actions/spec-sync). Use SpecSync as a reusable GitHub Action — no manual binary download needed.
=======
The easiest way to run SpecSync in CI. No manual binary download needed — the action auto-detects your runner's OS and architecture.
>>>>>>> fed5ec7 (docs: add GitHub Pages documentation site)

### Basic usage

```yaml
- uses: CorvidLabs/spec-sync@v1
  with:
    strict: 'true'
    require-coverage: '100'
```

### Inputs

| Input | Default | Description |
|-------|---------|-------------|
| `version` | `latest` | SpecSync release version to use |
| `strict` | `false` | Treat warnings as errors |
| `require-coverage` | `0` | Minimum file coverage percentage |
| `root` | `.` | Project root directory |
| `args` | `''` | Additional CLI arguments |

### Full workflow example

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

### Multi-platform matrix

```yaml
jobs:
  specsync:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: CorvidLabs/spec-sync@v1
        with:
          strict: 'true'
```

### Manual CI setup

If you prefer not to use the action:

```yaml
- name: Install specsync
  run: |
    curl -sL https://github.com/CorvidLabs/spec-sync/releases/latest/download/specsync-linux-x86_64.tar.gz | tar xz
    sudo mv specsync-linux-x86_64 /usr/local/bin/specsync

- name: Spec check
  run: specsync check --strict --require-coverage 100
```

---

## Configuration

Create `specsync.json` in your project root (or run `specsync init`):

```json
{
  "specsDir": "specs",
  "sourceDirs": ["src"],
  "schemaDir": "db/migrations",
  "requiredSections": [
    "Purpose",
    "Public API",
    "Invariants",
    "Behavioral Examples",
    "Error Cases",
    "Dependencies",
    "Change Log"
  ],
  "excludeDirs": ["__tests__"],
  "excludePatterns": ["**/__tests__/**", "**/*.test.ts", "**/*.spec.ts"],
  "sourceExtensions": []
}
```

### Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `specsDir` | `string` | `"specs"` | Directory containing `*.spec.md` files |
| `sourceDirs` | `string[]` | `["src"]` | Source directories to analyze for coverage |
| `schemaDir` | `string?` | — | SQL schema directory for `db_tables` validation |
| `schemaPattern` | `string?` | `CREATE TABLE` regex | Custom regex to extract table names from schema files |
| `requiredSections` | `string[]` | See above | Markdown sections every spec must include |
| `excludeDirs` | `string[]` | `["__tests__"]` | Directories excluded from coverage scanning |
| `excludePatterns` | `string[]` | Common test patterns | Glob patterns for files to exclude from coverage |
| `sourceExtensions` | `string[]` | All supported | Restrict analysis to specific file extensions (e.g., `["ts", "rs"]`) |

---

## How It Works

```
                    *.spec.md files
                         |
                    [1] Discover
                         |
                    [2] Parse frontmatter
                         |
              +----------+----------+
              |          |          |
         [3] Structure  [4] API   [5] Dependencies
              |          |          |
         - Required    - Detect   - depends_on exists?
           fields        language  - db_tables in schema?
         - File        - Extract  - Consumed By refs?
           exists?       exports
         - Required    - Compare
           sections?     with spec
              |          |          |
              +----------+----------+
                         |
                    [6] Report
                         |
              +----------+----------+
              |          |          |
           Errors    Warnings   Coverage %
```

1. **Discover** all `*.spec.md` files in your specs directory
2. **Parse** YAML frontmatter using a zero-dependency regex parser (no YAML library needed)
3. **Validate structure** — required fields, required markdown sections, file existence
4. **Validate API surface** — auto-detect language from extensions, extract exports, cross-reference against the spec's Public API tables
5. **Validate dependencies** — check `depends_on` specs exist, `db_tables` are in schema
6. **Report** errors, warnings, and coverage metrics

---

## Architecture

```
src/
├── main.rs            CLI entry point (clap) + output formatting
├── types.rs           Core data types + config schema
├── config.rs          specsync.json loading
├── parser.rs          YAML frontmatter + spec body parsing
├── validator.rs       Spec validation + coverage computation
├── generator.rs       Spec scaffolding for new modules
├── watch.rs           File watcher (notify crate, 500ms debounce)
└── exports/
    ├── mod.rs          Language dispatch + file utilities
    ├── typescript.rs   TypeScript/JS export extraction
    ├── rust_lang.rs    Rust pub item extraction
    ├── go.rs           Go exported identifier extraction
    ├── python.rs       Python __all__ / top-level extraction
    ├── swift.rs        Swift public/open item extraction
    ├── kotlin.rs       Kotlin public item extraction
    ├── java.rs         Java public item extraction
    ├── csharp.rs       C# public item extraction
    └── dart.rs         Dart public item extraction
```

### Design Principles

- **Single binary** — no runtime dependencies, no package managers, just download and run
- **Zero YAML dependencies** — frontmatter is parsed with regex, keeping the binary small
- **Language-agnostic architecture** — adding a new language means adding one file in `exports/`
- **Release-optimized** — LTO, symbol stripping, opt-level 3 for maximum performance

---

## For AI Agents

SpecSync is designed to work well with AI coding agents. Here's what you need to know:

- **`--json` flag** outputs structured results that are easy to parse programmatically
- **Exit code 1** means something needs attention; exit code 0 means all clear
- **`specsync generate`** can scaffold specs automatically — useful for bootstrapping docs on an existing codebase
- **Spec files are plain markdown** — any LLM can read and write them
- **The Public API table format** uses backtick-quoted names that are unambiguous to parse

### JSON output shape

```json
{
  "passed": false,
  "errors": ["auth.spec.md: phantom export `oldFunction` not found in source"],
  "warnings": ["auth.spec.md: undocumented export `newHelper`"],
  "specs_checked": 12
}
```

### Recommended AI workflow

```bash
# 1. Check current state
specsync check --json

# 2. Fix any errors (update specs or code)
# 3. Generate specs for new modules
specsync generate

# 4. Verify everything passes
specsync check --strict --require-coverage 100
```

---

## Contributing

1. Fork the repo
2. Create a feature branch (`git checkout -b feat/my-feature`)
3. Make your changes
4. Run tests: `cargo test`
5. Run lints: `cargo clippy`
6. Open a PR

### Adding a new language

1. Create `src/exports/yourlang.rs` implementing export extraction
2. Add the language variant to `Language` enum in `types.rs`
3. Wire it up in `src/exports/mod.rs`
4. Add tests for common export patterns

---

## License

[MIT](LICENSE) &copy; [CorvidLabs](https://github.com/CorvidLabs)
