---
module: cmd_report
version: 7
status: stable
files:
  - src/commands/report.rs
db_tables: []
tracks: []
depends_on:
  - specs/commands/commands.spec.md
  - specs/git_utils/git_utils.spec.md
  - specs/parser/parser.spec.md
  - specs/types/types.spec.md
  - specs/validator/validator.spec.md
---

# Cmd Report

## Purpose

Implements the `specsync report` command — a comprehensive per-module coverage report with staleness detection and completeness analysis. Uses git history to determine how many commits behind each spec is relative to its source files, and checks for missing frontmatter fields and empty sections.

## Public API

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_report` | `root: &Path, format: types::OutputFormat, stale_threshold: usize, exclude_status: &[String], only_status: &[String]` | `()` | Generate and display per-module coverage report with stale/incomplete detection |

## Invariants

7. Checked manifest discovery must succeed before project coverage is reported; malformed Gradle
   settings are inconclusive and exit 1.
8. A module's `stale` / `commits_behind` may be `false` / `0` ONLY when git actually measured them.
   With no committed history the pair is rendered as absent — `null` in JSON, `n/a` in text,
   markdown and github, empty in CSV — and the module is counted in `unmeasured_stale_modules`,
   never in `stale_modules` (#572).
9. The staleness guard is evaluated per module, at the point the verdict is written, and only for
   specs that list source files. A project whose specs list none never asks git anything and keeps
   its full report — the coverage half of the report is computed first and always renders.
10. Modules whose staleness could not be measured count as findings for the exit code, so `report`
    never exits 0 certifying "0 stale" over a question it did not ask. `enforcement = "warn"`
    still exits 0, with honest `n/a` cells.

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
| Git not available, not a git repo, or an unborn `HEAD` | Staleness is reported as UNMEASURED (`null` / `n/a` / empty), counted in `unmeasured_stale_modules`, explained by `staleness_error`, and gated like any other finding. It is never reported as 0 / not-stale (#572) |
| Spec lists no source files | Staleness was never relevant; the module reports a measured `false` / `0` and the report is unaffected by the absence of git |
| Spec references a file that doesn't exist | File is skipped in staleness calculation |
| No spec files found | Prints "no specs found" and exits 0 |
| Malformed Gradle settings prevent coverage discovery | Exits 1; JSON remains valid with `valid: false`, `inconclusive: true`, null overall coverage, zero counts, empty modules, and an explicit error |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| commands | `load_and_discover` |
| parser | `parse_frontmatter` |
| types | `OutputFormat` |
| validator | `compute_coverage_checked` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync report` subcommand |

## Change Log

| Date | Change |
|------|--------|
| 2026-07-22 | v3: fail closed when malformed Gradle/manifest discovery makes report coverage inconclusive |
| 2026-04-09 | Initial spec |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-27 | CHG-0063-close-independent-mcp-security-review-gaps-for-issue-414: Close independent MCP security review gaps for issue 414 |
| 2026-08-01 | CHG-0071-land-pre-6-0-product-fixes-for-hooks-init-coverage-naming-and-exit-codes-scoped: Land pre-6.0 product fixes for hooks init coverage naming and exit codes (scoped paths) |
| 2026-08-14 | CHG-0121-coverage-over-zero-source-files-must-report-nothing-measured-everywhere-replac: Coverage over zero source files must report nothing measured, everywhere: replace the precomputed percentage fields with Option-returning accessors so no renderer can substitute 100 percent for an unasked question |
| 2026-08-14 | CHG-0123-staleness-that-cannot-be-measured-must-be-refused-not-reported-as-zero-drift-i: Staleness that cannot be measured must be refused, not reported as zero drift, in every reader: report, check --stale, the lifecycle no_stale guard, and the score freshness dimension |
