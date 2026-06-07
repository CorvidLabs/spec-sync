---
spec: cmd_import.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/commands/import.rs` inline tests | Unit | Validate Cmd Import behavior close to implementation, especially `cmd_import`, `()`, `load_config`, `generate_companion_files`, `resolve_repo` |
| `tests/integration.rs` | Integration | Exercise Cmd Import through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Cmd Import contracts or source files.
- [ ] Run `fledge run test` and confirm Cmd Import unit/integration coverage still passes.
- [ ] Smoke-test `cargo run -- import --help` or the nearest CLI path that routes through this module.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Invalid source type | Exits 1 with supported list |
| Spec already exists | Exits 1 |
| Fetch fails | Exits 1 with error |
