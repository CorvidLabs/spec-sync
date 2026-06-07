---
spec: cmd_lifecycle.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/commands/lifecycle.rs` inline tests | Unit | Validate Cmd Lifecycle behavior close to implementation, especially `GuardResult`, `cmd_promote`, `()`, `cmd_demote`, `cmd_set`, `cmd_status`, `cmd_history`, `cmd_guard` |
| `tests/integration.rs` | Integration | Exercise Cmd Lifecycle through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Cmd Lifecycle contracts or source files.
- [ ] Run `fledge run test` and confirm Cmd Lifecycle unit/integration coverage still passes.
- [ ] Smoke-test `cargo run -- lifecycle --help` or the nearest CLI path that routes through this module.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Spec filter matches no specs | Exits 1 with error message |
| Ambiguous spec filter (multiple matches) | Exits 1, lists all matches |
| No `status:` line in frontmatter | Prints error, exits 1 |
| Invalid transition (without `--force`) | Prints error with valid alternatives, exits 1 |
