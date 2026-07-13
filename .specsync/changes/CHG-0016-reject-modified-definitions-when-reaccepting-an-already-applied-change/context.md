---
change: CHG-0016-reject-modified-definitions-when-reaccepting-an-already-applied-change
artifact: context
---

# Context

Audited reopen skips canonical delta application because the original acceptance already updated canonical truth. Review found that a caller could edit the reopened definition, approve and verify that new digest, and then reaccept successfully while those definition edits were silently ignored.

The correction binds reacceptance to the contract digest captured in the latest append-only reopen event. Further requirement or delta changes belong in a new change workspace, while delivery-only review fixes remain eligible for audited reopen.
