---
change: deps-reports-one-missing-dependency-as-one-error
artifact: testing
---

# Testing

Unit tests in `src/deps.rs` and the `regression_w1` integration test run against temp
projects. Both new assertions fail on the previous behavior.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| deps acceptance criterion (one error per missing dependency) | unit `test_one_missing_dependency_is_one_error`, `test_validate_missing_dep`; integration `deps_duplicate_depends_on_entries_deduped`, `deps_scalar_depends_on_is_not_silently_dropped` |
