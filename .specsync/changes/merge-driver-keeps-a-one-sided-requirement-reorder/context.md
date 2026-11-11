---
change: merge-driver-keeps-a-one-sided-requirement-reorder
artifact: context
---

# Context

#874 + t384 on #1058: merge_requirements seeded output order from the ancestor, silently dropping a one-sided reorder; and a reorder beside a deletion was masked by comparing full surviving sets. Order is compared over ids surviving on both sides; one-sided reorder wins, divergent reorders conflict, deletions keep their per-block rules. Cites REQ-merge-006 (clean only when the result keeps everything both sides wrote); no new requirement text (t370).
