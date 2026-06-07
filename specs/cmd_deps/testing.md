---
spec: cmd_deps.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/commands/deps.rs` inline tests | Unit | Validate Cmd Deps behavior close to implementation, especially `cmd_deps`, `()`, `validate_deps`, `load_config`, `OutputFormat` |
| `tests/integration.rs` | Integration | Exercise Cmd Deps through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Cmd Deps contracts or source files.
- [ ] Run `fledge run test` and confirm Cmd Deps unit/integration coverage still passes.
- [ ] Smoke-test `cargo run -- deps --help` or the nearest CLI path that routes through this module.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Circular dependency | Error printed, exits 1 |
| Missing dependency spec | Error printed, exits 1 |
| Empty dep graph | Prints hint about `depends_on` |
