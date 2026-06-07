---
spec: cmd_archive_tasks.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/commands/archive_tasks.rs` inline tests | Unit | Validate Cmd Archive Tasks behavior close to implementation, especially `cmd_archive_tasks`, `()`, `archive_tasks`, `load_config` |
| `tests/integration.rs` | Integration | Exercise Cmd Archive Tasks through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Cmd Archive Tasks contracts or source files.
- [ ] Run `fledge run test` and confirm Cmd Archive Tasks unit/integration coverage still passes.
- [ ] Smoke-test `cargo run -- archive-tasks --help` or the nearest CLI path that routes through this module.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| No tasks.md files found | Prints "nothing to archive" |
| No completed tasks | Prints "nothing to archive" |
