---
change: a-failed-lifecycle-state-history-index-build-is-retried-in-the-next-read-scope
artifact: tasks
---

# Tasks

- [x] Red-first unit test with a test-only failing-build hook: a failed build is not retried in its scope and is rebuilt in the next scope at the same HEAD
- [ ] Keep only built indexes in the per-thread slot; remember a failed build in the read scope
- [ ] Targeted fast-archived-read tests, fmt and clippy
