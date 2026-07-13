---
change: CHG-0015-add-audited-stale-accepted-change-reopening
artifact: design
---

# Design

`change reopen` accepts a change ID, explicit actor, and reason. Domain validation permits only `accepted` records whose current scoped delivery-input digest differs from the accepted digest while definition, closing approval, and commit/integration evidence remain otherwise valid.

The atomic transition appends `ReopenRecord` to `ApprovalLedger.reopenings`, embeds the untouched prior `VerificationRecord` and superseded `ApprovalRecord`, sets state to `verifying`, and preserves the existing verification file so strict checks remain stale. Fresh verification overwrites only the current evidence; history stays in the reopen event. Reacceptance appends a new closing approval and skips semantic application when `canonical_applied` is true.
