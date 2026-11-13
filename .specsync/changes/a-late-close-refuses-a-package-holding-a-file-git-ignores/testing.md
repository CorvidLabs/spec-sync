---
change: a-late-close-refuses-a-package-holding-a-file-git-ignores
artifact: testing
---

# Testing

- Integration `a_package_holding_an_ignored_file_is_not_closed_late` in `tests/integration/doctor.rs`
  (fails on main: the package is closed late).
- The existing late-close and doctor tests still pass.
