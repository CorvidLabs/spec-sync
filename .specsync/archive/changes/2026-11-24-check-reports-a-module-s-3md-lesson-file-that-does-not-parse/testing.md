---
change: check-reports-a-module-s-3md-lesson-file-that-does-not-parse
artifact: testing
---

# Testing

- Integration `an_unparseable_3md_is_a_check_warning_and_fails_strict` in
  `tests/integration/check.rs` (fails on main: no warning, and `--strict` passes).
- Integration `a_3md_over_the_read_bound_is_reported_and_not_read` in the same file (fails with a
  plain read: the file is read in full and nothing is reported), and
  `no_project_file_is_read_outside_the_bounded_read`, which names a plain read of the file.
- This repository's own `check --strict` still passes: every `.3md` here parses since #1062.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-cmd-check-023 | `tests/integration/check.rs`: `an_unparseable_3md_is_a_check_warning_and_fails_strict`, `a_3md_over_the_read_bound_is_reported_and_not_read` |
