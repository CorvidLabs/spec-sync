---
spec: cmd_issues.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/commands/issues.rs` inline tests | Unit | Validate Cmd Issues behavior close to implementation, especially `cmd_issues`, `()`, `load_config`, `parse_frontmatter`, `OutputFormat`, `find_spec_files`, `IgnoreRules` |
| `tests/integration.rs` | Integration | Exercise Cmd Issues through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Cmd Issues contracts or source files.
- [ ] Run `fledge run test` and confirm Cmd Issues unit/integration coverage still passes.
- [ ] Smoke-test `cargo run -- issues --help` or the nearest CLI path that routes through this module.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| GitHub repo unresolvable | Exits 1 with error message |
| `gh` CLI not available | API calls fail, counted as errors |
| Issue returns 404 | Counted as "not found", triggers non-zero exit |
| API rate limit | Counted as "error", reported but does not halt |
