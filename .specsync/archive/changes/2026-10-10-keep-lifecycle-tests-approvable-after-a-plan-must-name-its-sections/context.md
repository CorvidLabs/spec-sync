---
change: keep-lifecycle-tests-approvable-after-a-plan-must-name-its-sections
artifact: context
---

# Context

The plan gate now refuses a selected plan that does not name its sections. The lifecycle integration tests were still writing a one-line plan and then approving. Those edits landed in the pull request without being named on a change, so `change audit` failed after the earlier changes were archived.

This change claims those three test files and nothing else.
