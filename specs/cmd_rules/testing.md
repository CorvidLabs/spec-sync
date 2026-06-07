---
spec: cmd_rules.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/commands/rules.rs` inline tests | Unit | Validate Cmd Rules behavior close to implementation, especially `cmd_rules`, `()`, `print_builtin`, `load_config` |
| `tests/integration.rs` | Integration | Exercise Cmd Rules through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Cmd Rules contracts or source files.
- [ ] Run `fledge run test` and confirm Cmd Rules unit/integration coverage still passes.
- [ ] Smoke-test `cargo run -- rules --help` or the nearest CLI path that routes through this module.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Missing `specsync.json` | Config loader handles this (not this module's concern) |
