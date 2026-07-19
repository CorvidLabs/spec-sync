---
change: CHG-0052-allow-a-fully-valid-later-sequence-owner-to-preserve-historical-exact-ledger-evi
artifact: design
---

# Design

Add one narrow exception to recursive accepted-input validation. A changed exact input is eligible
only when it is `.specsync/change-sequence.json` and the current ledger owner is a later accepted or
archived record. The validator recursively authenticates that owner and its complete delivery
evidence before marking the historical ledger entry successor-covered.

No other exact input can use this path. The current ledger owner cannot cover itself, malformed
ledgers fail existing sequence validation, and recursive validation retains cycle detection and
memoization.
