---
spec: cmd_coverage.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/commands/coverage.rs` inline tests | Unit | Validate Cmd Coverage behavior close to implementation, especially `cmd_coverage`, `()`, `IgnoreRules::load`, `compute_coverage` |
| `tests/integration.rs` | Integration | Exercise Cmd Coverage through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Cmd Coverage contracts or source files.
- [ ] Run `fledge run test` and confirm Cmd Coverage unit/integration coverage still passes.
- [ ] Smoke-test `cargo run -- coverage --help` or the nearest CLI path that routes through this module.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Coverage below threshold | Exits 1 with details |
| No specs found | Prints suggestion, exits 0 |
