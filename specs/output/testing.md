---
spec: output.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/output.rs` inline tests | Unit | Validate Output behavior close to implementation, especially `print_summary`, `()`, `print_coverage_line`, `print_coverage_report`, `print_check_markdown`, `print_diff_markdown` |
| `tests/integration.rs` | Integration | Exercise Output through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Output contracts or source files.
- [ ] Run `fledge run test` and confirm Output unit/integration coverage still passes.
- [ ] Review examples in `output.spec.md` against observed behavior when touching src/output.rs.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Empty spec list | `print_summary` shows "0 passed, 0 failed" |
| Coverage report with no unspecced files | Shows "✓ All source files referenced by specs" |
| Diff with changed files not in any spec | Lists them under "Changed files not covered by any spec" |
