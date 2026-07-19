---
change: CHG-0053-permit-audited-reopened-collision-members-to-retain-immutable-sequence-history-s
artifact: design
---

# Design

Treat an active collision member as historical while it is in `verifying` only when all of these
conditions hold: canonical deltas were already applied, the latest reopen event targets the record,
the prior verification passed and still matches the definition, the referenced closing approval is
present and digest-valid, and the transition is exactly accepted-to-verifying.

This eligibility affects only collision acknowledgement validation. Normal accepted-input checks
still require fresh verification and closing approval before the record returns to `accepted`.
