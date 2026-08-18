---
change: CHG-0143-the-sequence-ledger-gate-must-judge-a-branch-by-its-own-history-not-by-origin
artifact: tasks
---

# Tasks

- [x] Reproduce the refusal on a branch that is behind origin with a valid ledger.
- [x] Establish that the gate prevents nothing there: allocation already floors
      against the remote mark, so the branch allocates past it either way.
- [x] Reject the merge-base comparison after finding it misses raise-then-rewrite.
- [x] Read the ledger's own history from HEAD in one `git log -p`, bounded, and
      take the maximum of added `"sequence"` lines.
- [x] Remove the origin oracle from the gate entirely, and the dead helper that
      documented an origin floor that was never wired.
- [x] Add the behind-branch test, the diverged-and-lowered control, and the
      raise-then-rewrite test that justifies the design.
- [x] Add the wiring test that fails when the write-side floor call is deleted.
- [x] Add the behind-branch assertions to sandbox drill 051 so the board catches
      this class mechanically rather than by review.
- [x] `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`.
- [x] Whole board unchanged at 48/7.
