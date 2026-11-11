---
change: a-lifecycle-rewrite-keeps-the-fields-a-newer-specsync-wrote
artifact: requirements
---

# Requirements

REQ-change-079 gains the rewrite half of the forward-compatibility valve: a lifecycle record or
ledger rewritten in place keeps the members this SpecSync does not know (see the delta). This makes
behavior match two existing criteria in `hi/change.md`: CHANGE-12.c ("Nothing already recorded
about the change is lost") and CHANGE-13.h ("an older SpecSync that drops it can still reopen and
finish the change"). Refusing to rewrite was considered and rejected because it contradicts
CHANGE-13.h (decided by the coordinator on 2026-11-11, relayed for Leif's review).
