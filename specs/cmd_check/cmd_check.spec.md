---
module: cmd_check
version: 12
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

Implements the primary deterministic validation entry point, including one fallible schema
snapshot, visible ignore suppression, caching, local Markdown auto-fix, structured formats, SDD
gates, and optional drift issues.

## Public API

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_check` | `root: &Path, strict: bool, enforcement: Option<types::EnforcementMode>, require_coverage: Option<usize>, format: types::OutputFormat, fix: bool, dry_run: bool, backup: bool, force: bool, create_issues: bool, explain: bool, stale: Option<Option<usize>>, spec_filters: &[String], exclude_status: &[String], only_status: &[String]` | `()` | Main check command: load config, discover specs, optionally bypass cache, run validation, auto-fix if requested, format output, exit with appropriate code |

## Invariants

9. Coverage uses checked manifest discovery; malformed Gradle settings make the result inconclusive
   and exit 1 instead of producing partial or vacuous coverage.
10. Text, JSON, Markdown, and GitHub output distinguish emitted warnings from deterministic
    suppressed-warning details, while strict exit behavior counts only unsuppressed findings.

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
| Malformed Gradle settings prevent coverage discovery | Emits an explicit inconclusive failure, preserves valid structured JSON in JSON mode, and exits 1 |

## Dependencies

**Consumes**

| Module | What is used |
|--------|-------------|
| commands | `load_and_discover`, `filter_specs`, `build_schema_columns`, `run_validation`, `compute_exit_code`, `exit_with_status`, `create_drift_issues` |
| hash_cache | `HashCache::load`, `save`, `is_changed` |
| ignore | `IgnoreRules::load` |
| output | `print_summary`, `print_coverage_line`, `print_check_markdown` |
| comment | `build_comment_body` |
| validator | `compute_coverage_checked`, `validate_spec` |
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
| 2026-07-22 | v9: fail closed when malformed Gradle/manifest discovery makes coverage inconclusive, preserving structured JSON failure output |
| 2026-07-10 | v5: add unified SDD lifecycle, approval, delta, effective-contract, and changed-path gates |
| 2026-06-11 | v4: `--fix` bypasses the hash cache (no more silent no-op after a cached warning run); bare API-kind headings are promoted to export headers and symbols already documented in any Public API table are not re-added; partial export-coverage summary prints as ⚠ so the warning count matches printed warnings |
| 2026-06-11 | v3: `--fix` routes exports to the matching table by kind; unmatched spec filters exit 1 without contradictory output |
| 2026-06-07 | Document generated review prompts for `--fix` export rows |
| 2026-04-09 | Initial spec |
| 2026-07-11 | CHG-0004-close-final-pr-review-gaps-in-5-0-lifecycle-enforcement: Close final PR review gaps in 5.0 lifecycle enforcement |
| 2026-07-11 | CHG-0007-harden-specsync-5-0-as-an-agent-native-secret-free-sdd-core-and-close-release-r: Harden SpecSync 5.0 as an agent-native, secret-free SDD core and close release regressions |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-27 | CHG-0063-close-independent-mcp-security-review-gaps-for-issue-414: Close independent MCP security review gaps for issue 414 |
| 2026-07-30 | CHG-0068-stabilize-specsync-6-0-with-a-low-churn-normal-workflow-preserved-audited-guara: Stabilize SpecSync 6.0 with one scope approval, same-PR finalization, lightweight archive CI, scoped review, and selected UX fixes |
| 2026-07-31 | CHG-0070-land-pre-6-0-product-fixes-for-hooks-init-coverage-naming-and-exit-codes: Land pre-6.0 product fixes for hooks init coverage naming and exit codes |
