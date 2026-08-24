---
change: bind-semantic-delta-bodies-to-the-approval-that-signed-them
artifact: tasks
---

# Tasks

- [x] Record a per-module delta body digest on the definition approval event
- [x] Refuse materialization and acceptance when an approved delta body changed
- [x] Read an absent digest as unknown so every archived change still validates
- [x] Keep the new field omitted when absent so no existing digest or ledger moves
- [x] Add the discriminator, the control and the compatibility test
- [x] Prove the discriminator fails and the other two pass with the check disabled
- [x] Correct spec invariant 3, add invariant 36 and `REQ-change-089`
- [x] Record the closed hole in `specs/change/context.md`
