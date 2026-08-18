---
change: CHG-0145-the-sequence-ledger-floor-must-be-wired-not-merely-present
artifact: context
---

# Context

`adbfb442` added `floor_sequence_ledger_to_committed` and called it from
`git_commit_all`, so every lifecycle commit raises a stale working-tree ledger
before `git add -A` stages it.

The function got three unit tests, including a proper vacuity control. None
asserted that anything CALLS it. Deleting the call at its single site left the
entire suite green while every lifecycle commit went back to staging a stale
ledger over a higher committed mark — the exact regression #533 exists to
prevent.

Tested machinery with an untested connection reads exactly like a working
feature. It is the same shape as the unwired mechanisms this release keeps
finding, except here it would have been introduced BY the fix rather than found
by it.

Discovered by an independent review of the #533 work, not by the suite.
