---
change: a-lifecycle-rewrite-keeps-the-fields-a-newer-specsync-wrote
artifact: tasks
---

# Tasks

- [x] Span parser, merge and splice that keep unknown members (src/change/unknown_members.rs)
- [x] Every in-place lifecycle rewrite uses it (state, approvals, attempts, review attempts, corrections, relay repair, archived state, sequence ledger)
- [x] Unit tests (older/newer record types) and the end-to-end CLI test; both fail on the old code
- [x] REQ-change-079 delta, invariant 52, the forward-compatibility comment and CHANGELOG
