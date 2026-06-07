---
spec: exports.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/exports/mod.rs` inline tests | Unit | Validate Exports behavior close to implementation, especially `get_exported_symbols`, `Vec<String>`, `get_exported_symbols_with_level`, `is_test_file`, `bool`, `is_source_file`, `has_extension`, `extract_exports` |
| `src/exports/typescript.rs` inline tests | Unit | Validate Exports behavior close to implementation, especially `get_exported_symbols`, `Vec<String>`, `get_exported_symbols_with_level`, `is_test_file`, `bool`, `is_source_file`, `has_extension`, `extract_exports` |
| `src/exports/python.rs` inline tests | Unit | Validate Exports behavior close to implementation, especially `get_exported_symbols`, `Vec<String>`, `get_exported_symbols_with_level`, `is_test_file`, `bool`, `is_source_file`, `has_extension`, `extract_exports` |
| `src/exports/rust_lang.rs` inline tests | Unit | Validate Exports behavior close to implementation, especially `get_exported_symbols`, `Vec<String>`, `get_exported_symbols_with_level`, `is_test_file`, `bool`, `is_source_file`, `has_extension`, `extract_exports` |
| `tests/integration.rs` | Integration | Exercise Exports through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Exports contracts or source files.
- [ ] Run `fledge run test` and confirm Exports unit/integration coverage still passes.
- [ ] Review examples in `exports.spec.md` against observed behavior when touching src/exports/mod.rs.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| File cannot be read | Returns empty vector |
| Unknown file extension | Returns empty vector |
| File has no exports | Returns empty vector |
| Binary or non-text file | Returns empty vector (read_to_string fails gracefully) |
