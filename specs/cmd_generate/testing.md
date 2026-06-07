---
spec: cmd_generate.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/commands/generate.rs` inline tests | Unit | Validate Cmd Generate behavior close to implementation, especially `cmd_generate`, `()`, `generate_spec_template`, `IgnoreRules::load`, `compute_coverage` |
| `tests/integration.rs` | Integration | Exercise Cmd Generate through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Cmd Generate contracts or source files.
- [ ] Run `fledge run test` and confirm Cmd Generate unit/integration coverage still passes.
- [ ] Smoke-test `cargo run -- generate --help` or the nearest CLI path that routes through this module.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| AI provider not found | Exits 1 |
| AI fails for one module | Error printed, continues |
| All modules already specced | Prints "all covered" |
