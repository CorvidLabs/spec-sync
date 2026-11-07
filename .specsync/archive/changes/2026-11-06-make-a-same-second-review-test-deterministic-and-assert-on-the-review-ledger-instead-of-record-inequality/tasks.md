---
change: make-a-same-second-review-test-deterministic-and-assert-on-the-review-ledger-instead-of-record-inequality
artifact: tasks
---

# Tasks

- [x] Test-only clock pin (`PinnedTestClock`, `#[cfg(test)]`), and the #969 unit test pinned to one second; fails every run.
- [x] Assert on the attempt ledger instead of record inequality.
- [x] Repeated pinned runs green; neighbouring tests green; verify.
