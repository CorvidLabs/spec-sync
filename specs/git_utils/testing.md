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
| `baseline_is_untracked_for_a_file_git_has_no_record_of` | `spec_baseline` returns `Untracked` for a never-committed path in a repo that HAS history |
| `baseline_is_a_commit_for_a_tracked_file` | Returns `Commit` with a full 40-char hex SHA for a tracked file |
| `baseline_separates_no_history_from_no_commit_for_this_path` | The distinction the type exists for: no repo → `Missing(NotARepository)`, unborn HEAD → `Missing(NoCommits)`, healthy repo + untracked spec → `Untracked` |
| `missing_history_reports_usable_history_as_none` | `None` only when there is history to measure against |
| `commits_since_is_zero_when_source_unchanged_after_spec` | `git_commits_since` is `0` when source has not changed since the spec commit |
| `commits_since_counts_source_changes_after_spec` | Counts exactly 3 source commits after the spec; an untouched file reports `0` |
| `commits_since_returns_zero_for_invalid_commit` | A bogus commit ref degrades to `0` (no panic) |
| `is_git_repo_detects_repo_and_non_repo` | `true` inside an initialized repo, `false` in a plain temp dir |

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Spec not tracked by git, repo has commits | a path that has never been committed | `spec_baseline` is called | returns `Untracked` |
| No history at all | a plain temp dir, and a repo with an unborn `HEAD` | `spec_baseline` is called | returns `Missing(NotARepository)` / `Missing(NoCommits)` — never `Untracked` |
| Source file changed after spec | a spec last committed at commit A, then 3 commits to a source file | `git_commits_since(root, A, source_file)` is called | returns `3` |
| Invalid spec commit | a bogus 40-zero commit ref | `git_commits_since` is called | returns `0` (graceful degradation) |
| Repo detection | initialized repo vs. plain temp dir | `is_git_repo` is called | `true` then `false` |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Not a git repository | `is_git_repo` returns false; other functions return safe defaults | Covered by `is_git_repo_detects_repo_and_non_repo` |
| Invalid / unresolvable commit ref | `git_commits_since` returns 0 | Covered by `commits_since_returns_zero_for_invalid_commit` |
| File not in git history | `spec_baseline` returns `Untracked` | Covered by `baseline_is_untracked_for_a_file_git_has_no_record_of` |
| No history at all vs. no commit for this path | The two must NEVER produce the same value — that conflation is #572 | Covered by `baseline_separates_no_history_from_no_commit_for_this_path`; end-to-end in `tests/integration/staleness_unmeasurable.rs` |

## Reviewer Checklist

- Run `cargo test git_utils` before the full suite when changing `src/git_utils.rs` (the tests shell out to real `git`).
- If you change the `git_commits_since` range or arguments, update `commits_since_counts_source_changes_after_spec` in the same commit.
- Keep the resolve-once / count-per-file contract: callers must not re-resolve the spec commit per source file (that was the removed N+1 path).
- Do NOT reintroduce a public `Option<String>` commit lookup. The private raw lookup plus `SpecBaseline` is the choke point that makes #572 a compile error rather than a review catch; a helper that callers must remember to call is fail-open by construction.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
