---
change: CHG-0021-close-reopened-lifecycle-review-gaps
artifact: design
---

# Design

Reuse one `ensure_reopened_definition_unchanged` guard in both strict checking
and closing. Keep canonical-applied records in `verifying` when their
definition is reapproved so fresh evidence remains mandatory. Anchor history
pathspecs at the repository top. Determine stale eligibility solely from the
accepted and current delivery-input digests after existing audit-integrity
preconditions pass.
