---
change: CHG-0017-allow-audited-reopen-after-squash-and-canonical-successors
artifact: design
---

# Design

Keep every existing fail-closed precondition: accepted state, unchanged definition digest, passed verification, valid closing approval, stale delivery inputs, and explicit human actor/reason. When the verification commit is unreachable and the workspace no longer matches the remote byte-for-byte, accept current-history evidence only if Git history records that change in accepted state, or if later recorded accepted/canonical changes govern every affected spec and path. Uncommitted or arbitrary off-history evidence remains invalid.
