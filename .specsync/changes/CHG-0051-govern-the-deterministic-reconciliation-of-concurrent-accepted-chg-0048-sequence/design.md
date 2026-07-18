---
change: CHG-0051-govern-the-deterministic-reconciliation-of-concurrent-accepted-chg-0048-sequence
artifact: design
---

# Design

Keep both accepted CHG-0048 directories byte-identifiable and add their sorted exact IDs to the
sequence ledger's collision acknowledgement. Advance the ledger to CHG-0051 and add one canonical
requirement documenting this integration rule. Do not change runtime code, replay prior semantic
deltas, or broaden any accepted change's delivery scope.
