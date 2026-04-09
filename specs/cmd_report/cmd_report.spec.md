---
module: cmd_report
version: 1
status: stable
files:
  - src/commands/report.rs
db_tables: []
tracks: []
depends_on:
  - specs/commands/commands.spec.md
  - specs/parser/parser.spec.md
  - specs/types/types.spec.md
  - specs/validator/validator.spec.md
---

# Cmd Report

## Purpose

Implements the `specsync report` command — a comprehensive per-module coverage report with staleness detection and completeness analysis. Uses git history to determine how many commits behind each spec is relative to its source files, and checks for missing frontmatter fields and empty sections.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_report` | `root: &Path, format: OutputFormat, stale_threshold: usize` | `()` | Generate and display per-module coverage report with stale/incomplete detection |

## Invariants

1. Staleness is measured by counting `git rev-list` commits between the spec's last-modified commit and each source file's latest commit
2. A spec is "stale" when any source file has `>= stale_threshold` commits ahead of the spec (default: 5)
3. Completeness checks: missing frontmatter fields (version, status, files) and empty required sections
4. Per-module coverage is the ratio of specced files to total source files in that module's directory
5. Text output uses a fixed-width table format with Module, Coverage, Stale, Incomplete columns
6. JSON output includes per-module detail arrays with stale commit counts and missing field lists

## Behavioral Examples

### Scenario: Stale spec detection

- **Given** `src/auth.rs` has 12 commits since `specs/auth/auth.spec.md` was last modified
- **When** `cmd_report` runs with default `stale_threshold: 5`
- **Then** auth module is flagged as stale with "12 commits behind"

### Scenario: All modules healthy

- **Given** all specs are up to date and complete
- **When** `cmd_report` runs
- **Then** every module shows "no" for Stale and Incomplete columns

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Git not available or not a git repo | Staleness detection gracefully returns 0 (not stale) |
| Spec references a file that doesn't exist | File is skipped in staleness calculation |
| No spec files found | Prints "no specs found" and exits 0 |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| commands | `load_and_discover` |
| parser | `parse_frontmatter` |
| types | `OutputFormat` |
| validator | `compute_coverage` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync report` subcommand |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-09 | Initial spec |
