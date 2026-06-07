---
spec: git_utils.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/git_utils.rs` inline tests | Unit | Validate Git Utils behavior close to implementation, especially `git_last_commit_hash`, `Option<String>`, `git_commits_between`, `usize`, `is_git_repo`, `bool`, `StaleInfo` |
| `tests/integration.rs` | Integration | Exercise Git Utils through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Git Utils contracts or source files.
- [ ] Run `fledge run test` and confirm Git Utils unit/integration coverage still passes.
- [ ] Review examples in `git_utils.spec.md` against observed behavior when touching src/git_utils.rs.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Not a git repository | `is_git_repo` returns false; other functions return safe defaults |
| Git not installed | All functions return None/0/false |
| File doesn't exist in git history | Returns None or 0 |
