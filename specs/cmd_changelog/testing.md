---
spec: cmd_changelog.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/commands/changelog.rs` inline tests | Unit | Validate Cmd Changelog behavior close to implementation, especially `cmd_changelog`, `()`, `generate_changelog`, `load_config`, `OutputFormat` |
| `tests/integration.rs` | Integration | Exercise Cmd Changelog through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Cmd Changelog contracts or source files.
- [ ] Run `fledge run test` and confirm Cmd Changelog unit/integration coverage still passes.
- [ ] Smoke-test `cargo run -- changelog --help` or the nearest CLI path that routes through this module.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Range missing `..` | Prints error and exits 1 |
| Invalid git refs | Git command fails, error propagated |
