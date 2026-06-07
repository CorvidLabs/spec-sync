---
spec: deps.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/deps.rs` inline tests | Unit | Validate Deps behavior close to implementation, especially `DepNode`, `DepsReport`, `build_dep_graph`, `validate_deps`, `extract_imports`, `HashSet<String>`, `format_report`, `String` |
| `tests/integration.rs` | Integration | Exercise Deps through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Deps contracts or source files.
- [ ] Run `fledge run test` and confirm Deps unit/integration coverage still passes.
- [ ] Review examples in `deps.spec.md` against observed behavior when touching src/deps.rs.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Source file unreadable | Skipped during import extraction |
| Spec frontmatter unparseable | Module excluded from dependency graph |
| No specs found in specs_dir | Returns empty graph and clean DepsReport |
