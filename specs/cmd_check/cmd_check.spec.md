---
module: cmd_check
version: 5
status: stable
files:
  - src/commands/check.rs
db_tables: []
tracks: []
depends_on:
  - specs/commands/commands.spec.md
  - specs/ai/ai.spec.md
  - specs/git_utils/git_utils.spec.md
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

1. When `--fix` is passed, auto-fix runs in two phases: (a) add undocumented exports to spec markdown tables with generated review prompts — type exports are routed to the "… Types" table and functions/values to the "… Functions"/"… Methods" table (falling back to the last export subsection), with rows padded to the target table's column count, (b) AI-regenerate specs whose requirements have drifted
2. Near-miss header correction runs as part of auto-fix — Levenshtein-close typos are renamed to canonical export headers, and bare API-kind headings under `## Public API` (e.g. `### Functions`, `### Methods`, `### Types`) are promoted to `### Exported <Kind>` so hand-written tables become the export table instead of being duplicated
3. Hash cache is consulted before validation unless `--force`, `--strict`, `--fix`, or a spec filter is set — an explicit `--fix` is never silently skipped because a previous failing/warning run recorded the hashes
3a. `--fix` never adds a symbol that already appears in any table within `## Public API` (including informational subsections)
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
- **Then** the export is appended to the matching Public API table (functions to the functions table, types to the types table) with a generated description prompt and the file is rewritten

### Scenario: JSON output format

- **Given** `--format json` is set
- **When** validation completes with errors and warnings
- **Then** output is a single JSON object with `specs_checked`, `passed`, `errors`, `warnings`, `coverage`, and `exit_code` fields

## Error Cases

| Condition | Behavior |
|-----------|----------|
| AI provider not available during `--fix` regen | Prints error per spec, continues with remaining specs |
| Auto-fix changes a spec but validation still fails | Reports remaining errors, does not loop |
| Spec name filter matches nothing while specs exist | Prints "No specs matched" error (no contradictory "No spec files found" message) and exits 1 |
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
| 2026-07-10 | v5: add unified SDD lifecycle, approval, delta, effective-contract, and changed-path gates |
| 2026-06-11 | v4: `--fix` bypasses the hash cache (no more silent no-op after a cached warning run); bare API-kind headings are promoted to export headers and symbols already documented in any Public API table are not re-added; partial export-coverage summary prints as ⚠ so the warning count matches printed warnings |
| 2026-06-11 | v3: `--fix` routes exports to the matching table by kind; unmatched spec filters exit 1 without contradictory output |
| 2026-06-07 | Document generated review prompts for `--fix` export rows |
| 2026-04-09 | Initial spec |
