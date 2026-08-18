---
module: cmd_stale
version: 6
status: stable
files:
  - src/commands/stale.rs
db_tables: []
tracks:
  - 188
depends_on:
  - specs/commands/commands.spec.md
  - specs/git_utils/git_utils.spec.md
  - specs/parser/parser.spec.md
  - specs/types/types.spec.md
---

# Cmd Stale

## Purpose

Implements the `specsync stale` command — a focused staleness detection tool that identifies specs whose source files have diverged via git commit history. Reports which specs need updating, how many commits they are behind, and which specific source files have drifted. Supports text, JSON, markdown, and GitHub output formats.

## Public API

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_stale` | `root: &Path, format: types::OutputFormat, threshold: usize, exclude_status: &[String], only_status: &[String]` | `()` | Detect and report stale specs based on git commit distance |

## Invariants

1. Staleness is determined by `git_commits_between`: a spec is stale when any source file has >= `threshold` commits since the spec was last modified (default: 5)
2. Specs with no `files` in frontmatter are counted as fresh (no source files to compare against)
3. Specs not yet tracked by git (no commit history) are skipped and counted as fresh
4. Results are sorted by most-stale-first (highest `max_commits_behind`)
5. Exit code is 1 when any stale specs are found, 0 when all are fresh
6. Requires a git repository — exits with error if `is_git_repo` returns false

## Behavioral Examples

### Scenario: All specs fresh

- **Given** all specs were updated after their source files
- **When** `specsync stale` is run
- **Then** prints "All specs are up to date" and exits 0

### Scenario: Spec behind source by 8 commits (threshold 5)

- **Given** module "auth" has source file `src/auth.rs` with 8 commits since spec was last updated
- **When** `specsync stale --threshold 5` is run
- **Then** reports auth as stale with "8 commits behind" and exits 1

### Scenario: JSON output

- **Given** 2 stale specs out of 10 total
- **When** `specsync stale --format json` is run
- **Then** outputs JSON with `total_specs: 10`, `stale_count: 2`, `stale_specs` array with per-file details

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Not a git repository | Prints error, exits 1 |
| Spec file unreadable | Skipped silently |
| No frontmatter | Skipped silently |
| Source file doesn't exist on disk | Skipped in commit distance check |

## Dependencies

- `git_utils` — git commit history queries
- `parser` — frontmatter parsing for module name and files list
- `commands` — `load_and_discover` for config and spec file discovery

## Change Log

| Date | Change |
|------|--------|
| 2026-04-10 | Initial — dedicated staleness detection command (closes #188) |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-08-01 | CHG-0071-land-pre-6-0-product-fixes-for-hooks-init-coverage-naming-and-exit-codes-scoped: Land pre-6.0 product fixes for hooks init coverage naming and exit codes (scoped paths) |
| 2026-08-13 | CHG-0113-staleness-detection-must-refuse-a-repository-with-no-commits-instead-of-reportin: Staleness detection must refuse a repository with no commits instead of reporting every spec current |
| 2026-08-14 | CHG-0123-staleness-that-cannot-be-measured-must-be-refused-not-reported-as-zero-drift-i: Staleness that cannot be measured must be refused, not reported as zero drift, in every reader: report, check --stale, the lifecycle no_stale guard, and the score freshness dimension |
| 2026-08-18 | CHG-0144-a-staleness-answer-must-not-read-an-unreadable-source-as-freshness: A staleness answer must not read an unreadable source as freshness |
