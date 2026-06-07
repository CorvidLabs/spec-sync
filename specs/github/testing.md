---
spec: github.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/github.rs` inline tests | Unit | Validate Github behavior close to implementation, especially `detect_repo`, `Option<String>`, `resolve_repo`, `gh_is_available`, `bool`, `fetch_issue_gh`, `fetch_issue_api`, `fetch_issue` |
| `tests/integration.rs` | Integration | Exercise Github through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Github contracts or source files.
- [ ] Run `fledge run test` and confirm Github unit/integration coverage still passes.
- [ ] Review examples in `github.spec.md` against observed behavior when touching src/github.rs.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| No git remote configured | `detect_repo` returns `None` |
| Neither config repo nor git remote | `resolve_repo` returns `Err` |
| `gh` unavailable and no `GITHUB_TOKEN` | `fetch_issue` returns `Err` |
| Issue does not exist (404) | `fetch_issue_api` returns `Err("Issue not found")` |
