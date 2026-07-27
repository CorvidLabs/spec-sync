---
module: git_utils
version: 3
status: stable
files:
  - src/git_utils.rs
db_tables: []
tracks: []
depends_on: []
---

# Git Utils

## Purpose

Shared git utility functions for querying repository history. Provides commit hash lookup, commit distance counting, and git repository detection. Used by the `stale`, `report`, `check`, `lifecycle`, and `scoring` modules to determine spec freshness relative to source file changes. Callers resolve a spec's commit hash once via `git_last_commit_hash`, then count divergence per source file via `git_commits_since` to avoid redundant `git log` invocations.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `git_last_commit_hash` | `root: &Path, file: &str` | `Option<String>` | Get the SHA hash of the last commit that touched a file |
| `git_commits_since` | `root: &Path, spec_commit: &str, source_file: &str` | `usize` | Count commits to source_file since a precomputed spec commit hash |
| `is_git_repo` | `root: &Path` | `bool` | Check if a directory is inside a git work tree |

### Exported Types

| Type | Kind | Description |
|------|------|-------------|
| `StaleInfo` | struct | Staleness summary for a single spec: path, module name, max commits behind, per-file details |

## Invariants

1. All git commands execute with `current_dir(root)` to ensure correct repository context
2. Functions return safe defaults (None, 0, false) when git is unavailable or commands fail
3. `git_commits_since` first compares source content with `spec_commit`, then uses `git rev-list --count {spec_commit}..HEAD -- {source_file}` only when bytes differ; the precomputed spec commit is resolved once per spec
4. `StaleInfo.source_details` only includes files with commits_behind > 0
5. Add/revert history that restores the source bytes at `spec_commit` reports zero drift

## Behavioral Examples

### Scenario: File not tracked by git

- **Given** a file that has never been committed
- **When** `git_last_commit_hash` is called
- **Then** returns `None`

### Scenario: Source file changed after spec

- **Given** a spec last committed at commit A, and a source file with 3 commits after A
- **When** `git_commits_since` is called with commit A's hash
- **Then** returns `3`

### Scenario: Source change is reverted

- **Given** a source changed after the spec and a later commit restores the exact prior bytes
- **When** `git_commits_since` compares it with the spec commit
- **Then** returns `0`

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Not a git repository | `is_git_repo` returns false; other functions return safe defaults |
| Git not installed | All functions return None/0/false |
| File doesn't exist in git history | Returns None or 0 |
| Intervening source commits net to byte-identical content | Returns 0 |

## Dependencies

None (only uses `std::process::Command` for git CLI calls).

## Change Log

| Date | Change |
|------|--------|
| 2026-07-26 | v3: make commit-distance freshness content-aware so add/revert history is not a false positive |
| 2026-04-10 | Initial — extracted from cmd_report for shared use by stale, report, and scoring |
| 2026-06-07 | Replaced `git_commits_between` with `git_commits_since`, which takes a precomputed spec commit hash so callers resolve it once per spec instead of once per source file (eliminates N+1 `git log` calls) |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
