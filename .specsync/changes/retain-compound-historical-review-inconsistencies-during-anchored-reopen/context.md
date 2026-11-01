---
change: retain-compound-historical-review-inconsistencies-during-anchored-reopen
artifact: context
---

# Context

A recorded historical review can disagree with both the accepted workspace and the claimed review hash. Reopen preserves that whole invalid review as untrusted evidence; normal acceptance and archive still refuse it. The exact remote-default package and all non-review integrity checks remain mandatory.
