---
change: keep-receipt-reads-bounded-and-recover-exactly-recorded-review-mismatches
artifact: design
---

# Design

Recursive accepted-input validation captures exact signed paths. Fresh acceptance and reopen retain full inventory and unchanged limits. A reopen-only fallback admits only an active v2 package whose raw files and modes equal the current remote-default tree, whose commit is an ancestor of HEAD, and whose other definition/execution/closing/review hashes remain valid. The optional audit anchor preserves the original package; the old mismatched review remains invalid. No serialized enum value is added.
