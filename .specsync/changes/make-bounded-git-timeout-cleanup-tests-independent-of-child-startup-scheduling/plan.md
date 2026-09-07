---
change: make-bounded-git-timeout-cleanup-tests-independent-of-child-startup-scheduling
artifact: plan
---

# Plan

1. Add a thread-local, test-only spawned-child observation seam beside existing test instrumentation.
2. Capture child.id() immediately after spawn; remove the shell-written PID dependency from the timeout cleanup regression.
3. Exercise deliberately delayed child startup while preserving the blocked stdin payload, deadline assertion, and process cleanup assertion.
4. Prove the original race and a cleanup defect with bounded negative controls; repeat focused tests.
5. Update the change spec and companions, run scoped and full verification, pre-push, trust and provenance.
6. Obtain implementation review, archive on the same PR, and merge only after required checks. Qualify a subsequent release candidate on merged main before any stable release.
