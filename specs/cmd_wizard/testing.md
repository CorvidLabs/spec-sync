---
spec: cmd_wizard.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/commands/wizard.rs` inline tests | Unit | Validate Cmd Wizard behavior close to implementation, especially `cmd_wizard`, `()`, `load_config` |
| `tests/integration.rs` | Integration | Exercise Cmd Wizard through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Cmd Wizard contracts or source files.
- [ ] Run `fledge run test` and confirm Cmd Wizard unit/integration coverage still passes.
- [ ] Smoke-test `cargo run -- wizard --help` or the nearest CLI path that routes through this module.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Empty module name entered | Exits with code 1 |
| Spec directory already exists | Prints error and exits 1 |
| User cancels at confirmation | Exits cleanly with code 0 |
| Directory creation fails | Exits with code 1 |
