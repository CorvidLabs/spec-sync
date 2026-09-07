---
change: make-bounded-git-timeout-cleanup-tests-independent-of-child-startup-scheduling
artifact: tasks
---

# Tasks

- [x] Capture child PID in parent-side test instrumentation.
- [x] Remove child startup scheduling dependency and retain cleanup assertions.
- [x] Run delayed-start and cleanup negative controls plus repeated focused tests.

Remaining lifecycle gates (executed after implementation tasks): materialize the approved spec delta with scoped check, complete full verification, obtain implementation review, archive, and merge with green required checks.
