---
spec: cmd_score.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/commands/score.rs` inline tests | Unit | Validate Cmd Score behavior close to implementation, especially `cmd_score`, `()`, `OutputFormat` |
| `tests/integration.rs` | Integration | Exercise Cmd Score through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Cmd Score contracts or source files.
- [ ] Run `fledge run test` and confirm Cmd Score unit/integration coverage still passes.
- [ ] Smoke-test `cargo run -- score --help` or the nearest CLI path that routes through this module.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| No specs match filters | Warning printed |
