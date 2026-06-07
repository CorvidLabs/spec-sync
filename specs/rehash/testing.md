---
spec: rehash.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/commands/rehash.rs` inline tests | Unit | Validate Rehash behavior close to implementation, especially `cmd_rehash`, `()`, `load_and_discover` |
| `tests/integration.rs` | Integration | Exercise Rehash through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Rehash contracts or source files.
- [ ] Run `fledge run test` and confirm Rehash unit/integration coverage still passes.
- [ ] Review examples in `rehash.spec.md` against observed behavior when touching src/commands/rehash.rs.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Cache save fails | Prints error, exits 1 |
