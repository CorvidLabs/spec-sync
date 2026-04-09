---
module: cmd_changelog
version: 1
status: stable
files:
  - src/commands/changelog.rs
db_tables: []
tracks: []
depends_on:
  - specs/changelog/changelog.spec.md
  - specs/config/config.spec.md
  - specs/types/types.spec.md
---

# Cmd Changelog

## Purpose

Implements the `specsync changelog` command. Generates a changelog of spec changes between two git refs, supporting text, JSON, and markdown output formats.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_changelog` | `root: &Path, range: &str, format: OutputFormat` | `()` | Generate and print a changelog of spec changes for a git ref range |

## Invariants

1. Range must contain `..` separator (e.g., `v0.1..v0.2`, `HEAD~5..HEAD`)
2. Delegates to `changelog::generate_changelog()` for git diff parsing
3. Exits with code 1 on invalid range format

## Behavioral Examples

### Scenario: Valid range with changes

- **Given** specs changed between `v3.5.0` and `v3.6.0`
- **When** `cmd_changelog(root, "v3.5.0..v3.6.0", Text)` is called
- **Then** prints list of added, modified, and removed specs

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Range missing `..` | Prints error and exits 1 |
| Invalid git refs | Git command fails, error propagated |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| changelog | `generate_changelog` |
| config | `load_config` |
| types | `OutputFormat` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync changelog` |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-09 | Initial spec |
