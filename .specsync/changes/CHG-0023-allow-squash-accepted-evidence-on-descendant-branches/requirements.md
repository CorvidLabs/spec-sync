---
change: CHG-0023-allow-squash-accepted-evidence-on-descendant-branches
artifact: requirements
---

# Requirements

- Closing validation must recognize an accepted state recorded in current Git history when the verification commit was replaced by a squash merge.
- Definition, delivery-input digest, passed verification, and matching closing approval checks remain mandatory before the history fallback.
- Complete current canonical successors remain an equivalent fallback for governed contract surfaces.
- Arbitrary off-history evidence with no recorded acceptance or complete successor governance continues to fail closed.
