---
spec: comment.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/comment.rs` inline tests | Unit | Validate Comment behavior close to implementation, especially `render_check_comment`, `String`, `detect_branch`, `Option<String>`, `CoverageReport` |
| `tests/integration.rs` | Integration | Exercise Comment through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Comment contracts or source files.
- [ ] Run `fledge run test` and confirm Comment unit/integration coverage still passes.
- [ ] Review examples in `comment.spec.md` against observed behavior when touching src/comment.rs.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Not in a git repository | `detect_branch` returns `None` |
| No repo/branch provided | Spec links use relative markdown format instead of GitHub URLs |
