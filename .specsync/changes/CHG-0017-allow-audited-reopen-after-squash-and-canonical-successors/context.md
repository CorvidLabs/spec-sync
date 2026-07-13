---
change: CHG-0017-allow-audited-reopen-after-squash-and-canonical-successors
artifact: context
---

# Context

Squash merging makes a valid verification commit unreachable. Later accepted changes can also supersede parts of an earlier governed workspace, so byte-for-byte comparison with the remote default branch is no longer sufficient proof that the earlier acceptance was canonical. Trust's migration records exercise both conditions.
