---
change: one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence
artifact: tasks
---

# Tasks

- [x] Failing tests: S1 and S2, scoped and unscoped, in `src/change_tests/terminal_evidence_isolation.rs`
- [x] Tests that a failure that counts still counts: a broken candidate successor, an unreadable `state.json` during a search
- [x] A test that a retired change whose replacement fails validation still reports that failure in its own status
- [x] Integration test: status, list, ship-status and audit --archives on the S1 fixture; check, review and finalize as on main
- [x] Per-change roster load for terminal evidence, and the three paths of `terminal_evidence_summary` on it
- [x] A failed candidate named in a successor search
- [x] CHANGELOG Fixed line
- [x] Rework: one constant for "changed after acceptance", shared by both changed-input writers and the successor search, with a failing test first
- [x] Rework: the binary test asserts plain `change audit` exits 1 with the error on a change open and archived at once, passes on S1, and `change audit --archives` names the change on S1
- [x] `fledge lanes run verify`, `change audit --strict` and the pre-push gate

Human gates and Chough's own records are not tasks here.
