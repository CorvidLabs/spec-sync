---
change: doctor-fix-refuses-under-an-inherited-verification-context-and-read-only-doctor-still-reports
artifact: testing
---

# Testing

- Integration `indirect_recursive_lifecycle_doctor_fix_fails_once_and_plain_doctor_reports` in
  `tests/integration/change.rs` (fails on main: `doctor --fix` runs under the marker).
- The existing recursion tests and the REQ-cli-003 contract pin still pass.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-cli-003 | `tests/integration/change.rs`: `indirect_recursive_lifecycle_doctor_fix_fails_once_and_plain_doctor_reports` |
