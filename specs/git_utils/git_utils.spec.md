---
module: git_utils
version: 6
status: stable
files:
  - src/git_utils.rs
db_tables: []
tracks: []
depends_on: []
---

# Git Utils

## Purpose

Shared git utility functions for querying repository history. Provides the staleness baseline for a spec, commit distance counting, and git repository detection. Used by the `stale`, `report`, `check`, `lifecycle`, and `scoring` modules to determine spec freshness relative to source file changes. Callers resolve a spec's baseline once via `spec_baseline`, then count divergence per source file via `git_commits_since` to avoid redundant `git log` invocations.

`spec_baseline` is the module's choke point. It replaced an `Option<String>` commit lookup that collapsed two unrelated absences into one `None`: "history exists, this spec simply is not in it" (drift is genuinely zero) and "there is no history at all" (drift is UNKNOWN). Four of the five callers read that `None` as the former and reported every spec current on a tree with no `.git`. The returned enum makes the distinction unspellable-away: every caller, including the next one written, is forced through an exhaustive `match` the compiler checks.

## Public API

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `spec_baseline` | `root: &Path, spec_file: &str` | `SpecBaseline` | Resolve the commit a spec's staleness is measured from, distinguishing an untracked spec from a tree with no history. Probes for missing history only when no commit came back, so a healthy repository still costs one `git log` per spec |
| `missing_history` | `root: &Path` | `Option<MissingHistory>` | Whether the tree has committed history to measure staleness against; `None` means history is usable |
| `git_commits_since` | `root: &Path, spec_commit: &str, source_file: &str` | `usize` | Count commits to source_file since a precomputed spec commit hash |
| `is_git_repo` | `root: &Path` | `bool` | Check if a directory is inside a git work tree |
| `has_commits` | `root: &Path` | `bool` | Whether the repository has any history. An unborn `HEAD` is a work tree by every other test, but nothing can be newer or older than a history that does not exist |

**Exported Types**

| Type | Kind | Description |
|------|------|-------------|
| `SpecBaseline` | enum | `Commit(String)` (measurable), `Untracked` (history exists, spec is not in it — drift is genuinely zero), `Missing(MissingHistory)` (drift is UNKNOWN and must never be reported as zero) |
| `MissingHistory` | enum | `NotARepository` or `NoCommits` — why a tree cannot be asked how far a spec has fallen behind |

**Exported Methods**

| Method | Parameters | Returns | Description |
|--------|-----------|---------|-------------|
| `reason` | `self: MissingHistory` | `&'static str` | Lowercase reason (`not a git repository` / `repository has no commits`), for machine payloads and mid-sentence use |
| `sentence` | `self: MissingHistory` | `&'static str` | Sentence-cased reason (`Not a git repository` / `Repository has no commits`), for the head of a human-readable error line |
| `StaleInfo` | struct | Staleness summary for a single spec: path, module name, max commits behind, per-file details |

## Invariants

1. All git commands execute with `current_dir(root)` to ensure correct repository context
2. Functions return safe defaults (None, 0, false) when git is unavailable or commands fail — except that "no history" is never one of those defaults: `spec_baseline` reports it as `Missing`, never as an absent commit
3. `git_commits_since` uses `git rev-list --count {spec_commit}..HEAD -- {source_file}` to count divergence, taking the spec commit hash as a parameter so it is resolved once per spec rather than once per source file
4. `StaleInfo.source_details` only includes files with commits_behind > 0
5. There is no public way to obtain a spec's commit hash without also learning whether the tree has any history: the raw lookup is private, and `SpecBaseline::Commit` is the only value that yields a hash

## Behavioral Examples

### Scenario: Spec not tracked by git, in a repository that has commits

- **Given** a repository with at least one commit and a spec that has never been committed
- **When** `spec_baseline` is called
- **Then** returns `SpecBaseline::Untracked` — there is nothing for the spec to be behind

### Scenario: No git history at all

- **Given** a directory that is not a git repository, or a repository with an unborn `HEAD`
- **When** `spec_baseline` is called
- **Then** returns `SpecBaseline::Missing(NotARepository)` or `SpecBaseline::Missing(NoCommits)` — the distance is unknown, not zero

### Scenario: Source file changed after spec

- **Given** a spec last committed at commit A, and a source file with 3 commits after A
- **When** `git_commits_since` is called with commit A's hash
- **Then** returns `3`

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Not a git repository | `is_git_repo` returns false; `missing_history` returns `Some(NotARepository)`; `spec_baseline` returns `Missing` |
| Repository with an unborn `HEAD` | `has_commits` returns false; `missing_history` returns `Some(NoCommits)`; `spec_baseline` returns `Missing` |
| Git not installed | Treated as "not a git repository": `spec_baseline` returns `Missing`, so callers refuse rather than report zero drift |
| File doesn't exist in git history | `spec_baseline` returns `Untracked`; `git_commits_since` returns 0 |

## Dependencies

None (only uses `std::process::Command` for git CLI calls).

## Change Log

| Date | Change |
|------|--------|
| 2026-04-10 | Initial — extracted from cmd_report for shared use by stale, report, and scoring |
| 2026-06-07 | Replaced `git_commits_between` with `git_commits_since`, which takes a precomputed spec commit hash so callers resolve it once per spec instead of once per source file (eliminates N+1 `git log` calls) |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-08-01 | CHG-0071-land-pre-6-0-product-fixes-for-hooks-init-coverage-naming-and-exit-codes-scoped: Land pre-6.0 product fixes for hooks init coverage naming and exit codes (scoped paths) |
| 2026-08-13 | CHG-0113-staleness-detection-must-refuse-a-repository-with-no-commits-instead-of-reportin: Staleness detection must refuse a repository with no commits instead of reporting every spec current |
| 2026-08-14 | #572: Made `git_last_commit_hash` private and replaced it with `spec_baseline -> SpecBaseline`, plus `missing_history -> Option<MissingHistory>`. The old `Option<String>` conflated "spec not in history" with "no history", and four of five callers read the second as the first |
| 2026-08-14 | CHG-0123-staleness-that-cannot-be-measured-must-be-refused-not-reported-as-zero-drift-i: Staleness that cannot be measured must be refused, not reported as zero drift, in every reader: report, check --stale, the lifecycle no_stale guard, and the score freshness dimension |
