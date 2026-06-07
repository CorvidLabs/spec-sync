---
spec: cmd_new.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/commands/new.rs` inline tests | Unit | Validate Cmd New behavior close to implementation, especially `cmd_new`, `()`, `load_config`, `generate_companion_files` |
| `tests/integration.rs` | Integration | Exercise Cmd New through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Cmd New contracts or source files.
- [ ] Run `fledge run test` and confirm Cmd New unit/integration coverage still passes.
- [ ] Smoke-test `cargo run -- new --help` or the nearest CLI path that routes through this module.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Spec already exists | Exits 1 |
| No source files found | Creates spec with empty `files:` |
| Dir creation fails | Exits 1 |
