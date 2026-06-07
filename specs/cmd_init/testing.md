---
spec: cmd_init.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/commands/init.rs` inline tests | Unit | Validate Cmd Init behavior close to implementation, especially `cmd_init`, `()`, `ensure_hashes_gitignored`, `detect_source_dirs` |
| `tests/integration.rs` | Integration | Exercise Cmd Init through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Cmd Init contracts or source files.
- [ ] Run `fledge run test` and confirm Cmd Init unit/integration coverage still passes.
- [ ] Smoke-test `cargo run -- init --help` or the nearest CLI path that routes through this module.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| File write fails | Exits 1 |
| No source dirs detected | Creates config with empty `sourceDirs` |
