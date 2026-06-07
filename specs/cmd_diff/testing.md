---
spec: cmd_diff.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/commands/diff.rs` inline tests | Unit | Validate Cmd Diff behavior close to implementation, especially `cmd_diff`, `()`, `load_and_discover`, `get_exported_symbols`, `print_diff_markdown`, `parse_frontmatter` |
| `tests/integration.rs` | Integration | Exercise Cmd Diff through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Cmd Diff contracts or source files.
- [ ] Run `fledge run test` and confirm Cmd Diff unit/integration coverage still passes.
- [ ] Smoke-test `cargo run -- diff --help` or the nearest CLI path that routes through this module.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| `git diff` fails (bad ref) | Exits with code 1 |
| Changed file not in any spec | Listed under "Changed files not covered by any spec" |
