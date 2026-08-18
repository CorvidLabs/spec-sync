---
change: CHG-0145-the-sequence-ledger-floor-must-be-wired-not-merely-present
artifact: tasks
---

# Tasks

- [x] Confirm the gap: the floor's own tests all pass with its call deleted.
- [x] Drive `git_commit_all` rather than the function it calls.
- [x] Assert on the committed ledger, not the working tree.
- [x] Verify the test fails with the call removed, from a tree where the removal
      was confirmed by grep rather than assumed.
- [x] `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`.
