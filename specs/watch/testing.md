---
spec: watch.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/watch.rs` inline tests | Unit | Validate Watch behavior close to implementation, especially `run_watch`, `()`, `load_config` |
| `tests/integration.rs` | Integration | Exercise Watch through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Watch contracts or source files.
- [ ] Run `fledge run test` and confirm Watch unit/integration coverage still passes.
- [ ] Review examples in `watch.spec.md` against observed behavior when touching src/watch.rs.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| No directories to watch | Prints error, exits with code 1 |
| Watcher creation fails | Panics with "Failed to create file watcher" |
| Individual dir watch fails | Prints warning, continues watching other dirs |
| Check command fails | Prints "Some checks failed", continues watching |
