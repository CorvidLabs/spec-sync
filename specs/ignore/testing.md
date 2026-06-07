---
spec: ignore.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/ignore.rs` inline tests | Unit | Validate Ignore behavior close to implementation, especially `WarningCategory`, `IgnoreRules`, `WarningCategory::from_str`, `Option<Self>`, `WarningCategory::classify`, `IgnoreRules::load`, `Self`, `IgnoreRules::parse_inline` |
| `tests/integration.rs` | Integration | Exercise Ignore through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Ignore contracts or source files.
- [ ] Run `fledge run test` and confirm Ignore unit/integration coverage still passes.
- [ ] Review examples in `ignore.spec.md` against observed behavior when touching src/ignore.rs.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| `.specsyncignore` does not exist | Returns empty `IgnoreRules` (not an error) |
| Unrecognized category string | Silently skipped during load; `from_str()` returns `None` |
| Malformed inline comment (missing `-->`) | Directive is ignored |
| Warning text doesn't match any pattern | `classify()` returns `None`, warning is never suppressed |
