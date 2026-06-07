---
spec: cmd_report.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/commands/report.rs` inline tests | Unit | Validate Cmd Report behavior close to implementation, especially `cmd_report`, `()`, `load_and_discover`, `parse_frontmatter`, `OutputFormat`, `compute_coverage` |
| `tests/integration.rs` | Integration | Exercise Cmd Report through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Cmd Report contracts or source files.
- [ ] Run `fledge run test` and confirm Cmd Report unit/integration coverage still passes.
- [ ] Smoke-test `cargo run -- report --help` or the nearest CLI path that routes through this module.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Git not available or not a git repo | Staleness detection gracefully returns 0 (not stale) |
| Spec references a file that doesn't exist | File is skipped in staleness calculation |
| No spec files found | Prints "no specs found" and exits 0 |
