---
spec: cmd_compact.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/commands/compact.rs` inline tests | Unit | Validate Cmd Compact behavior close to implementation, especially `cmd_compact`, `()`, `compact_changelogs`, `load_config` |
| `tests/integration.rs` | Integration | Exercise Cmd Compact through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Cmd Compact contracts or source files.
- [ ] Run `fledge run test` and confirm Cmd Compact unit/integration coverage still passes.
- [ ] Smoke-test `cargo run -- compact --help` or the nearest CLI path that routes through this module.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| No specs with changelogs | Prints "nothing to compact" |
| Fewer entries than keep | File unchanged |
