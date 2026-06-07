---
spec: generator.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/generator.rs` inline tests | Unit | Validate Generator behavior close to implementation, especially `generate_specs_for_unspecced_modules`, `usize`, `generate_specs_for_unspecced_modules_paths`, `Vec<String>`, `generate_companion_files_for_spec`, `()`, `find_files_for_module`, `generate_spec` |
| `tests/integration.rs` | Integration | Exercise Generator through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Generator contracts or source files.
- [ ] Run `fledge run test` and confirm Generator unit/integration coverage still passes.
- [ ] Review examples in `generator.spec.md` against observed behavior when touching src/generator.rs.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Cannot create spec directory | Prints error to stderr, skips module |
| Cannot write spec file | Prints error to stderr, skips module |
| AI generation fails | Falls back to template, prints warning |
| No source files found for module | Skips module entirely |
