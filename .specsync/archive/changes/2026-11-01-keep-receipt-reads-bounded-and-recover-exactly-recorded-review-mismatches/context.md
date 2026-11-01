---
change: keep-receipt-reads-bounded-and-recover-exactly-recorded-review-mismatches
artifact: context
---

# Context

Two acceptance-evidence failures can block archival: later assets can exhaust an old receipt read even though they were not signed inputs; a historically recorded finalization can have a review-workspace mismatch that cannot be reopened for fresh review. This change preserves strict acceptance and archival while giving only the exactly recorded invalid review a bounded downgrade path.
