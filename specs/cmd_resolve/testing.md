---
spec: cmd_resolve.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/commands/resolve.rs` inline tests | Unit | Validate Cmd Resolve behavior close to implementation, especially `cmd_resolve`, `()`, `load_and_discover`, `detect_repo` |
| `tests/integration.rs` | Integration | Exercise Cmd Resolve through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Cmd Resolve contracts or source files.
- [ ] Run `fledge run test` and confirm Cmd Resolve unit/integration coverage still passes.
- [ ] Smoke-test `cargo run -- resolve --help` or the nearest CLI path that routes through this module.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Local dep missing | Warning printed |
| Remote registry fetch fails | Warning, continues |
| Remote spec fetch fails | Warning, continues |
| Remote spec unparseable | Warning, continues |
