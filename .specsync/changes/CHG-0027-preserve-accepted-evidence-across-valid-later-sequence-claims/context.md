---
change: CHG-0027-preserve-accepted-evidence-across-valid-later-sequence-claims
artifact: context
---

# Context

Creating a valid later change advances `.specsync/change-sequence.json`. Historical accepted changes currently hash the old ledger bytes, so legitimate allocation can make their closing evidence appear stale.

The ledger remains protected exact evidence. Historical records should bind the canonical claim they owned, while the current owner must bind the exact current ledger and malformed or orphaned claims must fail closed.

The implementation will validate the complete current ledger before reconstructing the canonical historical claim for a predecessor. Source, contract, artifact, and every other covered input remain byte-exact evidence.
