---
module: cmd_check
version: 1
status: stable
files:
  - src/commands/check.rs
db_tables: []
tracks: []
depends_on:
  - specs/commands/commands.spec.md
  - specs/ai/ai.spec.md
  - specs/hash_cache/hash_cache.spec.md
  - specs/ignore/ignore.spec.md
  - specs/output/output.spec.md
  - specs/types/types.spec.md
  - specs/validator/validator.spec.md
  - specs/comment/comment.spec.md
  - specs/github/github.spec.md
---

# Cmd Check

## Purpose

Implements the `specsync check` command — the primary validation entry point. Validates all specs against source code, manages hash-based caching for incremental checks, supports auto-fix (adding undocumented exports, correcting near-miss headers, AI-regenerating stale specs), handles multiple output formats (text/json/markdown), and optionally creates GitHub drift issues.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_check` | `root, strict, enforcement, require_coverage, format, fix, force, create_issues, explain, spec_filters` | `()` | Main check command: load config, discover specs, optionally bypass cache, run validation, auto-fix if requested, format output, exit with appropriate code |

## Invariants

1. When `--fix` is passed, auto-fix runs in two phases: (a) add undocumented exports to spec markdown tables, (b) AI-regenerate specs whose requirements have drifted
2. Near-miss header correction (e.g., "Exported Functions" → "### Exported Functions") runs as part of auto-fix
3. Hash cache is consulted before validation unless `--force` is set — unchanged specs are skipped
4. After auto-fix, validation is re-run to verify fixes resolved the issues
5. JSON output mode collects all errors/warnings into a structured object instead of printing inline
6. `--create-issues` groups errors by spec path and creates one GitHub issue per affected spec
7. `--explain` appends per-category score breakdown (FM/Sec/API/Depth/Fresh each out of 20) to each spec's output
8. Exit code is determined by enforcement mode and `--strict` flag via `compute_exit_code`

## Behavioral Examples

### Scenario: Incremental check with cache

- **Given** 25 specs, 3 have changed since last check
- **When** `cmd_check` runs without `--force`
- **Then** only 3 specs are validated; 22 are skipped via hash cache

### Scenario: Auto-fix undocumented exports

- **Given** spec is missing export `pub fn new_function()`
- **When** `cmd_check` runs with `--fix`
- **Then** the export is appended to the spec's Public API table and the file is rewritten

### Scenario: JSON output format

- **Given** `--format json` is set
- **When** validation completes with errors and warnings
- **Then** output is a single JSON object with `specs_checked`, `passed`, `errors`, `warnings`, `coverage`, and `exit_code` fields

## Error Cases

| Condition | Behavior |
|-----------|----------|
| AI provider not available during `--fix` regen | Prints error per spec, continues with remaining specs |
| Auto-fix changes a spec but validation still fails | Reports remaining errors, does not loop |
| Hash cache file is corrupted | Falls back to full validation (cache miss) |
| `--create-issues` with no GitHub repo | Prints error, skips issue creation |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| commands | `load_and_discover`, `filter_specs`, `build_schema_columns`, `run_validation`, `compute_exit_code`, `exit_with_status`, `create_drift_issues` |
| ai | `resolve_ai_provider`, `regenerate_spec_with_ai` |
| hash_cache | `HashCache::load`, `save`, `is_changed` |
| ignore | `IgnoreRules::load` |
| output | `print_summary`, `print_coverage_line`, `print_check_markdown` |
| comment | `build_comment_body` |
| validator | `compute_coverage`, `validate_spec` |
| types | `SpecSyncConfig`, `OutputFormat`, `EnforcementMode`, `CoverageReport` |
| github | `resolve_repo` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync check` subcommand |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-09 | Initial spec |
