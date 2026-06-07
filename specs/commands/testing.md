---
spec: commands.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/commands/mod.rs` inline tests | Unit | Validate Commands behavior close to implementation, especially `load_and_discover`, `filter_specs`, `Vec<PathBuf>`, `filter_by_status`, `build_schema_columns`, `run_validation`, `compute_exit_code`, `i32` |
| `tests/integration.rs` | Integration | Exercise Commands through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Commands contracts or source files.
- [ ] Run `fledge run test` and confirm Commands unit/integration coverage still passes.
- [ ] Smoke-test `cargo run -- commands --help` or the nearest CLI path that routes through this module.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| No spec files found and `allow_empty` is false | Prints suggestion to run `specsync generate` and exits 0 |
| Filter matches no specs | Prints warning listing unmatched filters, returns empty vec |
| `schema_dir` not configured | `build_schema_columns` returns empty map (no error) |
| GitHub repo unresolvable for drift issues | Prints error and returns without creating issues |
