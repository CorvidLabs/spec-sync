---
change: retain-compound-historical-review-inconsistencies-during-anchored-reopen
artifact: research
---

# Research

A generic Git fixture creates both review inconsistencies with a valid finalization self-digest. The previous implementation refuses it at the review-hash check. The updated fixture proves recovery still requires a workspace mismatch and refuses a pure review-hash error, tampering, missing or unrelated remote evidence and an invalid finalization self-digest. No real consumer record is included.
