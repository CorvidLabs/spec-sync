---
spec: cmd_merge.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/commands/merge.rs` inline tests | Unit | Validate Cmd Merge behavior close to implementation, especially `cmd_merge`, `()`, `load_config`, `OutputFormat` |
| `tests/integration.rs` | Integration | Exercise Cmd Merge through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Cmd Merge contracts or source files.
- [ ] Run `fledge run test` and confirm Cmd Merge unit/integration coverage still passes.
- [ ] Smoke-test `cargo run -- merge --help` or the nearest CLI path that routes through this module.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| No conflicts | Prints "no conflicts" |
| Complex conflict | Exits 1 with file path |
