---
spec: schema.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/schema.rs` inline tests | Unit | Validate Schema behavior close to implementation, especially `SchemaColumn`, `SchemaTable`, `SpecColumn`, `column_names`, `Vec<&str>`, `build_schema`, `parse_spec_schema` |
| `tests/integration.rs` | Integration | Exercise Schema through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Schema contracts or source files.
- [ ] Run `fledge run test` and confirm Schema unit/integration coverage still passes.
- [ ] Review examples in `schema.spec.md` against observed behavior when touching src/schema.rs.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Schema directory does not exist | `build_schema` returns empty map |
| File cannot be read | File is silently skipped |
| Unmatched parentheses in CREATE TABLE | `extract_paren_body` returns `None`, table is skipped |
| No `### Schema` section in spec | `parse_spec_schema` returns empty map |
