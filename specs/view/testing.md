---
spec: view.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/view.rs` inline tests | Unit | Validate View behavior close to implementation, especially `view_spec`, `valid_roles` |
| `tests/integration.rs` | Integration | Exercise View through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing View contracts or source files.
- [ ] Run `fledge run test` and confirm View unit/integration coverage still passes.
- [ ] Review examples in `view.spec.md` against observed behavior when touching src/view.rs.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Unknown role string | Returns `Err` listing valid roles |
| Spec file unreadable | Returns `Err` with read error description |
| Frontmatter parse failure | Returns `Err` with parse error |
