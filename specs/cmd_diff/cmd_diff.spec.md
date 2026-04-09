---
module: cmd_diff
version: 1
status: stable
files:
  - src/commands/diff.rs
db_tables: []
tracks: []
depends_on:
  - specs/commands/commands.spec.md
  - specs/exports/exports.spec.md
  - specs/output/output.spec.md
  - specs/parser/parser.spec.md
  - specs/types/types.spec.md
---

# Cmd Diff

## Purpose

Implements the `specsync diff` command — shows which specs are affected by source file changes since a git base ref. Cross-references `git diff --name-only` output with spec frontmatter `files:` lists to identify affected specs, then computes export deltas (new/removed exports) for each.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_diff` | `root: &Path, base: &str, format: OutputFormat` | `()` | Show export changes per spec since a git base ref |

## Invariants

1. Uses `git diff --name-only {base}` to get the list of changed files
2. Only files matching `config.source_extensions` are considered
3. Export deltas are computed by comparing current exports against base-ref exports (via `git show`)
4. Changed files not covered by any spec are listed separately
5. Text output delegates to `output::print_diff_markdown`

## Behavioral Examples

### Scenario: New export added

- **Given** `src/auth.rs` added `pub fn verify_token()` since HEAD
- **When** `cmd_diff --base HEAD~1` runs
- **Then** shows `auth` spec with "Added: `verify_token`"

### Scenario: No spec-tracked changes

- **Given** only non-source files changed (e.g., README.md)
- **When** `cmd_diff` runs
- **Then** prints "No spec-tracked source files changed since `{base}`."

## Error Cases

| Condition | Behavior |
|-----------|----------|
| `git diff` fails (bad ref) | Exits with code 1 |
| Changed file not in any spec | Listed under "Changed files not covered by any spec" |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| commands | `load_and_discover` |
| exports | `get_exported_symbols` |
| output | `print_diff_markdown` |
| parser | `parse_frontmatter` |
| types | `OutputFormat`, `SpecSyncConfig` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync diff` subcommand |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-09 | Initial spec |
