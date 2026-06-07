---
spec: git_utils.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/git_utils.rs` | cargo test git_utils | No inline tests found; add focused coverage for `git_last_commit_hash`, `git_commits_between`, `usize`, `is_git_repo` before risky changes |

## Coverage Gaps

- Integration gap: add a fixture for "File not tracked by git" before changing user-visible CLI output, generated files, or error handling in git_utils.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| File not tracked by git | a file that has never been committed | `git_last_commit_hash` is called | returns `None` |
| Source file changed after spec | a spec last committed at commit A, and a source file with 3 commits after A | `git_commits_between` is called | returns `3` |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Not a git repository | `is_git_repo` returns false; other functions return safe defaults | Keep or add a focused assertion before changing this behavior |
| Git not installed | All functions return None/0/false | Keep or add a focused assertion before changing this behavior |
| File doesn't exist in git history | Returns None or 0 | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/git_utils.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
