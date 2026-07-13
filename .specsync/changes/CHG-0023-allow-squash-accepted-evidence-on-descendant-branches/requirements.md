---
change: CHG-0023-allow-squash-accepted-evidence-on-descendant-branches
artifact: requirements
---

# Requirements

- Closing validation must recognize an accepted state recorded on the remote default branch when the verification commit was replaced by a squash merge.
- Definition, delivery-input digest, passed verification, and matching closing approval checks remain mandatory before the history fallback.
- Arbitrary off-history evidence with no accepted state on the remote default branch continues to fail closed.
