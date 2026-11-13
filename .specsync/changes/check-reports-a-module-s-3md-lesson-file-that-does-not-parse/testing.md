---
change: check-reports-a-module-s-3md-lesson-file-that-does-not-parse
artifact: testing
---

# Testing

- Integration `an_unparseable_3md_is_a_check_warning_and_fails_strict` in
  `tests/integration/check.rs` (fails on main: no warning, and `--strict` passes).
- This repository's own `check --strict` still passes: every `.3md` here parses since #1062.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-cmd-check-018 | `tests/integration/check.rs`: `an_unparseable_3md_is_a_check_warning_and_fails_strict` |
