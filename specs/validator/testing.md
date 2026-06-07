---
spec: validator.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/validator.rs` inline tests | Unit | Validate Validator behavior close to implementation, especially `validate_spec`, `ValidationResult`, `find_spec_files`, `Vec<PathBuf>`, `compute_coverage`, `CoverageReport`, `get_schema_table_names`, `HashSet<String>` |
| `tests/integration.rs` | Integration | Exercise Validator through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Validator contracts or source files.
- [ ] Run `fledge run test` and confirm Validator unit/integration coverage still passes.
- [ ] Review examples in `validator.spec.md` against observed behavior when touching src/validator.rs.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Spec file unreadable | Error: "Cannot read spec" |
| Missing frontmatter delimiters | Error: "Missing or malformed YAML frontmatter" |
| Source file not found | Error with fix suggestion (Levenshtein-based or removal) |
| DB table not in schema | Error: "DB table not found in schema" |
