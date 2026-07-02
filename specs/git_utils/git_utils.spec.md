---
module: git_utils
version: 2
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
| `is_file_tracked` | `root: &Path, rel_path: &str` | `bool` | Check whether a path is tracked by git (committed or staged); false when untracked or not a git repo |

### Exported Types

| Type | Kind | Description |
|------|------|-------------|
| `StaleInfo` | struct | Staleness summary for a single spec: path, module name, max commits behind, per-file details |

## Invariants

1. All git commands execute with `current_dir(root)` to ensure correct repository context
2. Functions return safe defaults (None, 0, false) when git is unavailable or commands fail
3. `git_commits_since` uses `git rev-list --count {spec_commit}..HEAD -- {source_file}` to count divergence, taking the spec commit hash as a parameter so it is resolved once per spec rather than once per source file
4. `StaleInfo.source_details` only includes files with commits_behind > 0

## Behavioral Examples

### Scenario: File not tracked by git

- **Given** a file that has never been committed
- **When** `git_last_commit_hash` is called
- **Then** returns `None`

### Scenario: Source file changed after spec

- **Given** a spec last committed at commit A, and a source file with 3 commits after A
- **When** `git_commits_since` is called with commit A's hash
- **Then** returns `3`

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Not a git repository | `is_git_repo` returns false; other functions return safe defaults |
| Git not installed | All functions return None/0/false |
| File doesn't exist in git history | Returns None or 0 |

## Dependencies

None (only uses `std::process::Command` for git CLI calls).

## Change Log

| Date | Change |
|------|--------|
| 2026-04-10 | Initial — extracted from cmd_report for shared use by stale, report, and scoring |
| 2026-06-07 | Replaced `git_commits_between` with `git_commits_since`, which takes a precomputed spec commit hash so callers resolve it once per spec instead of once per source file (eliminates N+1 `git log` calls) |
| 2026-07-01 | Added `is_file_tracked` — used by config loading to honor a `config.local.toml` `ai_command` only when the file is untracked (closes the H1 committed-`config.local.toml` shell-injection path) |
