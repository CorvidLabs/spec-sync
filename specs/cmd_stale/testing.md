---
spec: cmd_stale.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/commands/stale.rs` inline tests | Unit | Validate Cmd Stale behavior close to implementation, especially `cmd_stale`, `()` |
| `tests/integration.rs` | Integration | Exercise Cmd Stale through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Cmd Stale contracts or source files.
- [ ] Run `fledge run test` and confirm Cmd Stale unit/integration coverage still passes.
- [ ] Smoke-test `cargo run -- stale --help` or the nearest CLI path that routes through this module.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Not a git repository | Prints error, exits 1 |
| Spec file unreadable | Skipped silently |
| No frontmatter | Skipped silently |
| Source file doesn't exist on disk | Skipped in commit distance check |
