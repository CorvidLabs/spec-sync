# SpecSync

[![GitHub Marketplace](https://img.shields.io/badge/Marketplace-SpecSync%20Check-blue?logo=github)](https://github.com/marketplace/actions/specsync-check)

Bidirectional spec-to-code validation. Keep your module specs and source code in sync with CI-enforced contract checking.

**Written in Rust. Language-agnostic. Blazing fast.**

> **Now available on the [GitHub Marketplace](https://github.com/marketplace/actions/specsync-check)** — add spec validation to any repo in one step.

## What it does

SpecSync validates that your markdown module specifications match your actual source code — in both directions:

- **Code exports something not in the spec?** Warning: undocumented export
- **Spec documents something that doesn't exist?** Error: stale spec entry
- **Source file referenced in spec was deleted?** Error: missing file
- **DB table in spec doesn't exist in schema?** Error: phantom table
- **Required section missing from spec?** Error: incomplete spec

## Supported Languages

| Language | Export Detection | Test File Exclusion |
|----------|----------------|---------------------|
| TypeScript/JavaScript | `export function`, `export class`, `export type`, `export const`, `export enum`, re-exports | `.test.ts`, `.spec.ts`, `.d.ts` |
| Rust | `pub fn`, `pub struct`, `pub enum`, `pub trait`, `pub type`, `pub const`, `pub static`, `pub mod` | (inline tests, no file exclusion) |
| Swift | `public`/`open` func, class, struct, enum, protocol, typealias, actor, init | `*Tests.swift`, `*Test.swift` |
| Kotlin | Top-level declarations (public by default), excludes `private`/`internal`/`protected` | `*Test.kt`, `*Spec.kt` |
| Java | `public` class, interface, enum, record, methods, fields | `*Test.java`, `*Tests.java` |
| Go | Uppercase identifiers: `func Name`, `type Name`, `var Name`, `const Name`, methods | `_test.go` |
| Python | `__all__` list, or top-level `def`/`class` (excluding `_` prefixed) | `test_*.py`, `*_test.py` |
| C# | `public` class, struct, interface, enum, record, delegate, methods | `*Test.cs`, `*Tests.cs` |
| Dart | Top-level declarations (public = no `_` prefix), class, mixin, enum, typedef | `*_test.dart` |

Language is auto-detected from file extensions. The same spec format works for any language.

## Install

### GitHub Action (recommended)

Available on the [GitHub Marketplace](https://github.com/marketplace/actions/specsync-check). Add to any workflow:

```yaml
- uses: CorvidLabs/spec-sync@v1
  with:
    strict: 'true'
    require-coverage: '100'
```

No binary download or Rust toolchain needed — the action handles everything.

### Pre-built binaries

Download the latest binary from [GitHub Releases](https://github.com/CorvidLabs/spec-sync/releases):

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

Windows: download `specsync-windows-x86_64.exe.zip` from the releases page.

### From source

```bash
cargo install --git https://github.com/CorvidLabs/spec-sync

# Or clone and build
git clone https://github.com/CorvidLabs/spec-sync.git
cd spec-sync
cargo install --path .
```

## Quick start

```bash
# Create a config file
specsync init

# Validate all specs
specsync check

# See coverage report
specsync coverage

# Generate specs for unspecced modules
specsync generate
```

## Spec format

Specs are markdown files with YAML frontmatter:

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

Handles authentication and session management.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `authenticate` | `(token: string)` | `User \| null` | Validates a token |

### Exported Types

| Type | Description |
|------|-------------|
| `User` | Authenticated user object |

## Invariants

1. Sessions expire after 24 hours
2. Failed auth attempts are rate-limited

## Behavioral Examples

### Scenario: Valid token

- **Given** a valid JWT token
- **When** `authenticate()` is called
- **Then** returns the corresponding User

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Expired token | Returns null, logs warning |
| Malformed token | Returns null |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| database | `query()` for user lookups |

## Change Log

| Date | Author | Change |
|------|--------|--------|
| 2026-03-18 | team | Initial spec |
```

## Configuration

Create a `specsync.json` in your project root:

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

| Option | Default | Description |
|--------|---------|-------------|
| `specsDir` | `"specs"` | Directory containing spec files |
| `sourceDirs` | `["src"]` | Source directories for coverage |
| `schemaDir` | — | SQL schema dir for `db_tables` validation |
| `schemaPattern` | `CREATE TABLE` regex | Pattern to extract table names |
| `requiredSections` | Standard set | Required markdown sections |
| `excludeDirs` | `["__tests__"]` | Dirs excluded from coverage |
| `excludePatterns` | test files | File patterns excluded from coverage |
| `sourceExtensions` | all supported | Restrict to specific extensions (e.g., `["ts", "rs"]`) |

## CLI

```
specsync [command] [flags]

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
```

## GitHub Action

Available on the [GitHub Marketplace](https://github.com/marketplace/actions/specsync-check). Use SpecSync as a reusable GitHub Action — no manual binary download needed.

### Basic usage

```yaml
- uses: CorvidLabs/spec-sync@v1
  with:
    strict: 'true'
    require-coverage: '100'
```

### All options

```yaml
- uses: CorvidLabs/spec-sync@v1
  with:
    version: 'latest'        # SpecSync version (default: latest)
    strict: 'true'           # Treat warnings as errors (default: false)
    require-coverage: '100'  # Minimum file coverage % (default: 0)
    root: '.'                # Project root directory (default: .)
    args: ''                 # Additional CLI arguments (default: '')
```

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

The action automatically detects the runner OS and architecture (x86_64 and aarch64 on Linux/macOS, x86_64 on Windows) and downloads the correct pre-built binary.

## CI integration (manual)

```yaml
# GitHub Actions — install from release binary
- name: Install specsync
  run: |
    curl -sL https://github.com/CorvidLabs/spec-sync/releases/latest/download/specsync-linux-x86_64.tar.gz | tar xz
    sudo mv specsync-linux-x86_64 /usr/local/bin/specsync

- name: Spec check
  run: specsync check --strict --require-coverage 100
```

```yaml
# Or install from source (slower but always up to date)
- name: Install specsync
  run: cargo install --git https://github.com/CorvidLabs/spec-sync

- name: Spec check
  run: specsync check --strict --require-coverage 100
```

## How it works

1. **Discovers** all `*.spec.md` files in your specs directory
2. **Parses** YAML frontmatter (zero-dependency regex parser, no YAML library)
3. **Validates structure** — required fields, required sections, file existence
4. **Validates API surface** — auto-detects language, parses exports, cross-references against spec's Public API tables
5. **Validates dependencies** — checks that `depends_on` spec files exist
6. **Reports coverage** — which source files and modules have specs

## Architecture

```
src/
├── main.rs           CLI (clap) + output formatting
├── types.rs          Core data types + config schema
├── config.rs         specsync.json loading
├── parser.rs         Frontmatter + spec body parsing
├── validator.rs      Spec validation + coverage computation
├── generator.rs      Spec scaffolding for new modules
└── exports/
    ├── mod.rs        Language dispatch + file utilities
    ├── typescript.rs TypeScript/JS export extraction
    ├── rust_lang.rs  Rust pub item extraction
    ├── swift.rs      Swift public/open item extraction
    ├── kotlin.rs     Kotlin public item extraction
    ├── java.rs       Java public item extraction
    ├── go.rs         Go exported identifier extraction
    ├── python.rs     Python __all__ / top-level extraction
    ├── csharp.rs     C# public item extraction
    └── dart.rs       Dart public item extraction
```

## License

MIT
