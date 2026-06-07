---
spec: cmd_init_registry.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/commands/init_registry.rs` inline tests | Unit | Validate Cmd Init Registry behavior close to implementation, especially `cmd_init_registry`, `()`, `load_config`, `generate_registry` |
| `tests/integration.rs` | Integration | Exercise Cmd Init Registry through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Cmd Init Registry contracts or source files.
- [ ] Run `fledge run test` and confirm Cmd Init Registry unit/integration coverage still passes.
- [ ] Smoke-test `cargo run -- init-registry --help` or the nearest CLI path that routes through this module.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Registry exists | Early return |
| Write fails | Exits 1 |
