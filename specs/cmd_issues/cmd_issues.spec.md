---
module: cmd_issues
version: 1
status: stable
files:
  - src/commands/issues.rs
db_tables: []
tracks: []
depends_on:
  - specs/commands/commands.spec.md
  - specs/config/config.spec.md
  - specs/github/github.spec.md
  - specs/parser/parser.spec.md
  - specs/types/types.spec.md
  - specs/validator/validator.spec.md
  - specs/ignore/ignore.spec.md
---

# Cmd Issues

## Purpose

Implements the `specsync issues` command — verifies GitHub issue references in spec frontmatter (`implements:`, `tracks:` fields) against the GitHub API. Reports valid, closed, not-found, and errored references. Optionally creates drift issues for specs with validation errors.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_issues` | `root: &Path, format: OutputFormat, create: bool` | `()` | Verify issue references across all specs and optionally create drift issues |

## Invariants

1. Checks both `implements` and `tracks` frontmatter fields for issue numbers
2. Each issue number is verified via GitHub API (`gh api repos/{owner}/{repo}/issues/{num}`)
3. Counts are tallied: valid (open), closed, not found (404), error (API failure)
4. With `--create`, calls `create_drift_issues` for specs with validation errors
5. Exits 1 if any issue references are not found (404)

## Behavioral Examples

### Scenario: All references valid

- **Given** specs reference issues #10, #15, #20 — all exist and are open
- **When** `cmd_issues` runs
- **Then** prints "3 valid, 0 closed, 0 not found" and exits 0

### Scenario: Stale reference

- **Given** spec references issue #5 which was deleted
- **When** `cmd_issues` runs
- **Then** prints error for issue #5 and exits 1

## Error Cases

| Condition | Behavior |
|-----------|----------|
| GitHub repo unresolvable | Exits 1 with error message |
| `gh` CLI not available | API calls fail, counted as errors |
| Issue returns 404 | Counted as "not found", triggers non-zero exit |
| API rate limit | Counted as "error", reported but does not halt |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| commands | `build_schema_columns`, `run_validation`, `create_drift_issues` |
| config | `load_config` |
| github | `resolve_repo`, GitHub API calls |
| parser | `parse_frontmatter` |
| types | `OutputFormat` |
| validator | `find_spec_files` |
| ignore | `IgnoreRules` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync issues` subcommand |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-09 | Initial spec |
