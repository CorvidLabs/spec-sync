---
change: ship-readiness-is-a-content-question-not-a-history-one
artifact: docs
---

# Docs

`docs/ADOPTING.md` currently tells adopters to expect a re-verify after every squash-merge, and
that no configuration avoids it (#692, open).

That guidance is now half wrong in the reader's favour and must be updated when this lands:
readiness no longer breaks on a squash. What still breaks is the scoped review, so the honest
revision is narrower rather than removed — expect a fresh review after a squash, not a full
re-verification.

Deliberately not changed here: #692 is still open and editing the same paragraph from two branches
would conflict. The revision belongs in whichever lands second.
