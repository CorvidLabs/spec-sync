---
change: CHG-0051-govern-the-deterministic-reconciliation-of-concurrent-accepted-chg-0048-sequence
artifact: docs
---

# Docs

Add `REQ-change-034` to the canonical change contract. It documents that concurrent accepted
sequence collisions are reconciled without renumbering immutable histories: the collision set is
exact and sorted, and a later governed sequence claim owns the merged ledger transition.
