---
change: an-excluded-source-file-never-names-a-module-that-coverage-asks-a-spec-for
artifact: testing
---

# Testing

- Unit `an_excluded_flat_source_file_names_no_module` in `src/validator/tests.rs` (fails on main:
  `generated` is reported as an unspecced module).
- The existing coverage tests still pass.
