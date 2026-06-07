---
spec: parser.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/parser.rs` inline tests | Unit | Validate Parser behavior close to implementation, especially `ParsedSpec`, `parse_frontmatter`, `Option<ParsedSpec>`, `get_spec_symbols`, `Vec<String>`, `get_missing_sections`, `is_export_header`, `bool` |
| `tests/integration.rs` | Integration | Exercise Parser through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Parser contracts or source files.
- [ ] Run `fledge run test` and confirm Parser unit/integration coverage still passes.
- [ ] Review examples in `parser.spec.md` against observed behavior when touching src/parser.rs.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| No frontmatter delimiters | `parse_frontmatter` returns `None` |
| Malformed YAML in frontmatter | Unknown keys silently ignored, missing fields remain as `None` |
| No `## Public API` section | `get_spec_symbols` returns empty vector |
| Empty body | `get_missing_sections` reports all required sections as missing |
