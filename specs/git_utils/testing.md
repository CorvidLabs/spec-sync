---
spec: git_utils.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/git_utils.rs` | cargo test git_utils | Inline `#[cfg(test)]` module using `tempfile` + real `git` invocations (see fixtures below) |

### Inline test fixtures (all in `src/git_utils.rs`)

| Test | Verifies |
|------|----------|
| `last_commit_hash_returns_none_for_untracked_file` | `git_last_commit_hash` returns `None` for a never-committed path |
| `last_commit_hash_returns_hash_for_tracked_file` | Returns a full 40-char hex SHA for a tracked file |
| `commits_since_is_zero_when_source_unchanged_after_spec` | `git_commits_since` is `0` when source has not changed since the spec commit |
| `commits_since_counts_source_changes_after_spec` | Counts exactly 3 source commits after the spec; an untouched file reports `0` |
| `commits_since_returns_zero_for_invalid_commit` | A bogus commit ref degrades to `0` (no panic) |
| `is_git_repo_detects_repo_and_non_repo` | `true` inside an initialized repo, `false` in a plain temp dir |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| File not tracked by git | a path that has never been committed | `git_last_commit_hash` is called | returns `None` |
| Source file changed after spec | a spec last committed at commit A, then 3 commits to a source file | `git_commits_since(root, A, source_file)` is called | returns `3` |
| Invalid spec commit | a bogus 40-zero commit ref | `git_commits_since` is called | returns `0` (graceful degradation) |
| Repo detection | initialized repo vs. plain temp dir | `is_git_repo` is called | `true` then `false` |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Not a git repository | `is_git_repo` returns false; other functions return safe defaults | Covered by `is_git_repo_detects_repo_and_non_repo` |
| Invalid / unresolvable commit ref | `git_commits_since` returns 0 | Covered by `commits_since_returns_zero_for_invalid_commit` |
| File not in git history | `git_last_commit_hash` returns None | Covered by `last_commit_hash_returns_none_for_untracked_file` |
| Add/revert source history restores the spec-commit bytes | `git_commits_since` returns 0 | Covered by the add/revert freshness regression |

## Reviewer Checklist

- Run `cargo test git_utils` before the full suite when changing `src/git_utils.rs` (the tests shell out to real `git`).
- If you change the `git_commits_since` range or arguments, update `commits_since_counts_source_changes_after_spec` in the same commit.
- Keep the resolve-once / count-per-file contract: callers must not re-resolve the spec commit per source file (that was the removed N+1 path).
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
