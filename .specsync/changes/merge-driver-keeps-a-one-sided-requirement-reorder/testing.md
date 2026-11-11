---
change: merge-driver-keeps-a-one-sided-requirement-reorder
artifact: testing
---

# Testing

Unit tests in merge_driver.rs: one-sided reorder is kept in both directions;
divergent reorders produce a conflicted merge.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-merge-007 | merge_driver unit tests |
