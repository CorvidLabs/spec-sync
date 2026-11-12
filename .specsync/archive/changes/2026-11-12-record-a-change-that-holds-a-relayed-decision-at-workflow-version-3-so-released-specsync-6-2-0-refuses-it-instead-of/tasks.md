---
change: record-a-change-that-holds-a-relayed-decision-at-workflow-version-3-so-released-specsync-6-2-0-refuses-it-instead-of
artifact: tasks
---

# Tasks

- [x] Version 3 load, anchor and history rules; the `== 2` finalization check reads `>= 2`
- [x] Execution digest reads version 3 as 2
- [x] Approve and review raise a v2 change to 3 before writing a relay; on v1 they keep version 1 and print the note
- [x] Every other `state.json` write raises a v2 change whose ledgers hold a relay
- [x] Unit and integration tests, including a downgrade refusal and unchanged digests
- [x] `.github/scripts/test-relay-gate-compat.sh` with pinned 6.2.0, as a fledge task and in CI
- [x] Spec invariant 60, Change Log and version; `change.3md` lesson; docs; CHANGELOG
