---
spec: cmd_comment.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/commands/comment.rs` inline tests | Unit | Validate Cmd Comment behavior close to implementation, especially `cmd_comment`, `()`, `build_comment_body`, `resolve_repo` |
| `tests/integration.rs` | Integration | Exercise Cmd Comment through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Cmd Comment contracts or source files.
- [ ] Run `fledge run test` and confirm Cmd Comment unit/integration coverage still passes.
- [ ] Smoke-test `cargo run -- comment --help` or the nearest CLI path that routes through this module.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| `gh` CLI not installed | Command fails with error |
| GitHub repo unresolvable | Exits 1 |
