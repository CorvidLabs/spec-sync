---
module: cmd_issues
version: 4
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
2. References from all specs are sent through one globally deduplicated, capped, and time-bounded
   GitHub verification batch.
3. Counts are tallied: valid (open), closed, not found (404), error (API failure)
4. Human-readable output prints no-reference guidance only when no spec references were gathered;
   all-error batches print a summary with the error count.
5. With `--create`, calls `create_drift_issues` for specs with validation errors
6. Exits 1 if any issue references are not found (404) or unverifiable
7. Specs are scanned before repository/provider resolution; an empty reference set succeeds with
   no-reference guidance and performs no GitHub access.

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
| No references and no configured/detectable repository | Prints no-reference guidance and exits 0 without provider access |
| `gh` CLI not available | API calls fail, counted as errors |
| Issue returns 404 | Counted as "not found", triggers non-zero exit |
| API rate limit | Counted as "error", reported but does not halt |
| Repository inaccessible or provider malformed/timed out | Counted as error, never not-found |
| More than 100 unique issue IDs | Batch error before provider access |
| Every referenced issue is unverifiable | Prints an error-count summary, never no-reference guidance, and exits 1 |

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
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-22 | CHG-0063: Batch, deduplicate, cap, fail closed, and report all-error GitHub verification truthfully |
| 2026-07-22 | CHG-0063: Skip repository and provider resolution when no issue references are present |
