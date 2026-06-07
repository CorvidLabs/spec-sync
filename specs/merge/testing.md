---
spec: merge.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/merge.rs` inline tests | Unit | Validate Merge behavior close to implementation, especially `merge_specs`, `Vec<MergeResult>`, `has_conflict_markers`, `bool`, `print_results`, `()`, `results_to_json`, `String` |
| `tests/integration.rs` | Integration | Exercise Merge through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Merge contracts or source files.
- [ ] Run `fledge run test` and confirm Merge unit/integration coverage still passes.
- [ ] Review examples in `merge.spec.md` against observed behavior when touching src/merge.rs.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Spec file unreadable | Marked as `Manual` with read error in details |
| `git diff` command fails | Falls back to scanning all files for conflict markers |
| Post-resolution frontmatter invalid | Warning printed; file is still written with resolved content |
