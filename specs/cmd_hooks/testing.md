---
spec: cmd_hooks.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/commands/hooks.rs` inline tests | Unit | Validate Cmd Hooks behavior close to implementation, especially `cmd_hooks`, `()` |
| `tests/integration.rs` | Integration | Exercise Cmd Hooks through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Cmd Hooks contracts or source files.
- [ ] Run `fledge run test` and confirm Cmd Hooks unit/integration coverage still passes.
- [ ] Smoke-test `cargo run -- hooks --help` or the nearest CLI path that routes through this module.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Hook write fails | Delegated to hooks module |
