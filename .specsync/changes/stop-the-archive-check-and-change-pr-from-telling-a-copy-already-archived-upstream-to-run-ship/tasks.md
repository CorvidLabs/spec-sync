---
change: stop-the-archive-check-and-change-pr-from-telling-a-copy-already-archived-upstream-to-run-ship
artifact: tasks
---

# Tasks

- [x] Rust unit test (stale copy with a passing review at HEAD); fails first on main.
- [x] Script test case in test-archive-readiness.sh; fails first on main.
- [x] `archive_candidate_can_ship` consults `upstream_archive_of`.
- [x] `archive-readiness.sh` mirrors the upstream-archive test.
- [x] REQ-cmd-change-029 delta, CHANGELOG, lesson.
