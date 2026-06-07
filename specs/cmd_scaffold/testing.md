---
spec: cmd_scaffold.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/commands/scaffold.rs` inline tests | Unit | Validate Cmd Scaffold behavior close to implementation, especially `cmd_add_spec`, `()`, `cmd_scaffold`, `load_config`, `get_exported_symbols`, `generate_companion_files`, `append_to_registry` |
| `tests/integration.rs` | Integration | Exercise Cmd Scaffold through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Cmd Scaffold contracts or source files.
- [ ] Run `fledge run test` and confirm Cmd Scaffold unit/integration coverage still passes.
- [ ] Smoke-test `cargo run -- scaffold --help` or the nearest CLI path that routes through this module.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Spec exists | Early return |
| Dir creation fails | Exits 1 |
| Custom template dir missing | Falls back to built-in |
