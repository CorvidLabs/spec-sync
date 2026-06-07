---
spec: changelog.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/changelog.rs` inline tests | Unit | Validate Changelog behavior close to implementation, especially `FieldChange`, `ModifiedSpec`, `ChangelogReport`, `SpecEntry`, `parse_range`, `generate_changelog`, `format_text`, `String` |
| `tests/integration.rs` | Integration | Exercise Changelog through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Changelog contracts or source files.
- [ ] Run `fledge run test` and confirm Changelog unit/integration coverage still passes.
- [ ] Review examples in `changelog.spec.md` against observed behavior when touching src/changelog.rs.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| Git ref doesn't exist | `list_specs_at_ref` returns empty list — no crash |
| Range string has no `..` separator | `parse_range` returns `None` |
| Spec frontmatter unparseable at a ref | Spec is silently skipped in diff |
