---
spec: cmd_view.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/commands/view.rs` inline tests | Unit | Validate Cmd View behavior close to implementation, especially `cmd_view`, `()`, `load_config`, `find_spec_files`, `view_spec` |
| `tests/integration.rs` | Integration | Exercise Cmd View through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Cmd View contracts or source files.
- [ ] Run `fledge run test` and confirm Cmd View unit/integration coverage still passes.
- [ ] Smoke-test `cargo run -- view --help` or the nearest CLI path that routes through this module.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| No specs found | Exits 1 |
| Spec read error | Error printed, continues |
