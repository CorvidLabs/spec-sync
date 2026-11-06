---
change: require-a-fresh-review-after-a-send-back-even-when-the-scope-comes-back-unchanged
artifact: tasks
---

# Tasks

- [x] Integration test from the #960 scenario; fails first on main 8a251d7a.
- [x] Unit tests on the real ledger, including the legacy timestamp fallback.
- [x] `SendBackRecord.cancelled_review_attempts`, written on every new send-back.
- [x] Review currency refuses a review recorded before the latest send-back.
- [x] REQ-change-123 delta, spec invariant and Public API row, CHANGELOG, HI CHANGE-12.e, lesson.
