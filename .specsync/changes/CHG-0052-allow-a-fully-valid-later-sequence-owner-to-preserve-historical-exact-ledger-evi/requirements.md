---
change: CHG-0052-allow-a-fully-valid-later-sequence-owner-to-preserve-historical-exact-ledger-evi
artifact: requirements
---

# Requirements

The implementation SHALL fulfill existing REQ-change-029 when a collision acknowledgement is
introduced after predecessor acceptance.

- Only `.specsync/change-sequence.json` is eligible for automatic later-owner coverage.
- The owner must be the current validated maximum claim and sort after the predecessor.
- The owner must have closing-valid authenticated accepted or archived evidence.
- All non-ledger inputs and the current owner's ledger input remain exact.
