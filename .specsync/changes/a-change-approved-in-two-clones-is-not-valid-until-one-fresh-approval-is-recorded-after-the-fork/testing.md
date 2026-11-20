---
change: a-change-approved-in-two-clones-is-not-valid-until-one-fresh-approval-is-recorded-after-the-fork
artifact: testing
---

# Testing

- **Unit** (`src/change_tests/forked_approvals.rs`): a fork across two branches merged `-s ours` is refused, naming both approvers and the fresh approval, and status reports the approval as not valid; one fresh approval settles it. Plus the guards: approvals that only append, a send-back and fresh approval, and a relay removed or restored are not a fork, and an archived change is not judged again.
- **Through the binary** (`tests/integration/forked_approvals.rs`): two real clones of one origin, each approves and commits, then `git merge -X ours`. `change check` and `change audit --strict` refuse, naming both approvers and both commits, and `change status` says the approval is not valid. One `change approve` lets `change check --commit` pass.
- **Gates:** `change audit --strict` with origin/main fetched, and `fledge lanes run verify`.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-169 | `src/change_tests/forked_approvals.rs`; `tests/integration/forked_approvals.rs` |
