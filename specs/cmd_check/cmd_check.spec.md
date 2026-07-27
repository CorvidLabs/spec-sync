---
module: cmd_check
version: 9
status: stable
files:
  - src/commands/check.rs
db_tables: []
tracks: []
depends_on:
  - specs/commands/commands.spec.md
  - specs/comment/comment.spec.md
  - specs/config/config.spec.md
  - specs/git_utils/git_utils.spec.md
  - specs/github/github.spec.md
  - specs/hash_cache/hash_cache.spec.md
  - specs/ignore/ignore.spec.md
  - specs/output/output.spec.md
  - specs/parser/parser.spec.md
  - specs/types/types.spec.md
  - specs/util/util.spec.md
  - specs/validator/validator.spec.md
---

# Cmd Check

## Purpose

Implements the primary deterministic validation entry point, including caching, local markdown auto-fix, output formats, SDD gates, and optional drift issues.

## Public API

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_check` | `root: &Path, strict: bool, enforcement: Option<types::EnforcementMode>, require_coverage: Option<usize>, format: types::OutputFormat, fix: bool, dry_run: bool, backup: bool, force: bool, create_issues: bool, explain: bool, stale: Option<Option<usize>>, spec_filters: &[String], exclude_status: &[String], only_status: &[String]` | `()` | Main check command: load config, discover specs, optionally bypass cache, run validation, auto-fix if requested, format output, exit with appropriate code |

## Invariants

1. `--fix` performs deterministic local markdown repairs only: near-miss headers and undocumented export rows; it never calls a model or shell command
2. Near-miss header correction runs as part of auto-fix — Levenshtein-close typos are renamed to canonical export headers, and bare API-kind headings under `## Public API` (e.g. `### Functions`, `### Methods`, `### Types`) are promoted to `### Exported <Kind>` so hand-written tables become the export table instead of being duplicated
3. Hash cache is consulted before validation unless `--force`, `--strict`, `--fix`, or a spec filter is set — an explicit `--fix` is never silently skipped because a previous failing/warning run recorded the hashes
3a. `--fix` never adds a symbol that already appears in any table within `## Public API` (including informational subsections)
4. After auto-fix, validation is re-run to verify fixes resolved the issues
5. JSON output mode collects all errors/warnings into a structured object instead of printing inline
6. `--create-issues` groups errors by spec path and creates one GitHub issue per affected spec
7. `--explain` appends per-category score breakdown (FM/Sec/API/Depth/Fresh each out of 20) to each spec's output
8. Exit code is determined by enforcement mode and `--strict` flag via `compute_exit_code`
9. The effective configured enforcement mode is resolved before the SDD gate; warn-mode lifecycle findings remain visible but nonblocking unless `--strict` is explicit

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
| Auto-fix changes a spec but validation still fails | Reports remaining errors, does not loop |
| Spec name filter matches nothing while specs exist | Prints "No specs matched" error (no contradictory "No spec files found" message) and exits 1 |
| Hash cache file is corrupted | Falls back to full validation (cache miss) |
| `--create-issues` with no GitHub repo | Prints error, skips issue creation |
| SDD findings under configured warn mode | Prints the findings and continues canonical validation with exit 0 unless another gate fails |

## Dependencies

**Consumes**

| Module | What is used |
|--------|-------------|
| commands | `load_and_discover`, `filter_specs`, `build_schema_columns`, `run_validation`, `compute_exit_code`, `exit_with_status`, `create_drift_issues` |
| hash_cache | `HashCache::load`, `save`, `is_changed` |
| ignore | `IgnoreRules::load` |
| output | `print_summary`, `print_coverage_line`, `print_check_markdown` |
| comment | `build_comment_body` |
| validator | `compute_coverage`, `validate_spec` |
| types | `SpecSyncConfig`, `OutputFormat`, `EnforcementMode`, `CoverageReport` |
| github | `resolve_repo` |

**Consumed By**

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync check` subcommand |

**Frontmatter Synchronization**

Implementation SHALL add these canonical dependency specs to `depends_on`: `specs/config/config.spec.md`, `specs/parser/parser.spec.md`, `specs/util/util.spec.md`. This YAML frontmatter update is an explicit implementation edit because semantic section deltas do not apply frontmatter.

## Change Log

| Date | Change |
|------|--------|
| 2026-07-26 | v9: apply configured warn enforcement to the early SDD gate so advisory lifecycle findings do not become false failures |
| 2026-07-10 | v5: add unified SDD lifecycle, approval, delta, effective-contract, and changed-path gates |
| 2026-06-11 | v4: `--fix` bypasses the hash cache (no more silent no-op after a cached warning run); bare API-kind headings are promoted to export headers and symbols already documented in any Public API table are not re-added; partial export-coverage summary prints as ⚠ so the warning count matches printed warnings |
| 2026-06-11 | v3: `--fix` routes exports to the matching table by kind; unmatched spec filters exit 1 without contradictory output |
| 2026-06-07 | Document generated review prompts for `--fix` export rows |
| 2026-04-09 | Initial spec |
| 2026-07-11 | CHG-0004-close-final-pr-review-gaps-in-5-0-lifecycle-enforcement: Close final PR review gaps in 5.0 lifecycle enforcement |
| 2026-07-11 | CHG-0007-harden-specsync-5-0-as-an-agent-native-secret-free-sdd-core-and-close-release-r: Harden SpecSync 5.0 as an agent-native, secret-free SDD core and close release regressions |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
