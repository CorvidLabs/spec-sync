---
spec: cmd_check.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/commands/check.rs` inline tests | Unit | Validate Cmd Check behavior close to implementation, especially `cmd_check`, `()`, `IgnoreRules::load`, `build_comment_body`, `resolve_repo` |
| `tests/integration.rs` | Integration | Exercise Cmd Check through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Cmd Check contracts or source files.
- [ ] Run `fledge run test` and confirm Cmd Check unit/integration coverage still passes.
- [ ] Smoke-test `cargo run -- check --help` or the nearest CLI path that routes through this module.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| AI provider not available during `--fix` regen | Prints error per spec, continues with remaining specs |
| Auto-fix changes a spec but validation still fails | Reports remaining errors, does not loop |
| Hash cache file is corrupted | Falls back to full validation (cache miss) |
| `--create-issues` with no GitHub repo | Prints error, skips issue creation |
