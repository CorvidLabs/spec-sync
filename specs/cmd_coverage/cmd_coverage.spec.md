---
module: cmd_coverage
version: 1
status: stable
files:
  - src/commands/coverage.rs
db_tables: []
tracks: []
depends_on:
  - specs/commands/commands.spec.md
  - specs/output/output.spec.md
  - specs/ignore/ignore.spec.md
  - specs/types/types.spec.md
  - specs/validator/validator.spec.md
---

# Cmd Coverage

## Purpose

Implements the `specsync coverage` command. Reports file-level and LOC-level spec coverage with support for JSON and text output.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_coverage` | `root, strict, enforcement, require_coverage, format` | `()` | Compute and display spec coverage report |

## Invariants

1. Runs validation before computing coverage for accurate results
2. JSON output includes coverage percentages, file counts, and unspecced file list
3. Delegates exit code to `exit_with_status()`

## Behavioral Examples

### Scenario: Full coverage

- **Given** all source files claimed by specs
- **When** `cmd_coverage` runs
- **Then** prints 100% with green check marks

### Scenario: Below threshold

- **Given** 58% coverage, `--require-coverage 80`
- **When** `cmd_coverage` runs
- **Then** lists uncovered files and exits 1

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Coverage below threshold | Exits 1 with details |
| No specs found | Prints suggestion, exits 0 |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| commands | `load_and_discover`, `build_schema_columns`, `run_validation`, `exit_with_status` |
| output | `print_coverage_line`, `print_coverage_report` |
| ignore | `IgnoreRules::load` |
| validator | `compute_coverage` |
| types | `OutputFormat`, `EnforcementMode` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync coverage` |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-09 | Initial spec |
