---
module: merge
---

## ADDED

### REQUIREMENT REQ-merge-007

When one side of a requirements merge reorders existing requirement blocks and
the other side keeps the ancestor's order, the merge SHALL keep the reordered
sequence. When both sides reorder differently, the merge SHALL be conflicted
rather than silently choosing one ordering.

#### Acceptance Criteria

- A one-sided reorder survives the merge in either merge direction.
- Divergent reorders produce a conflicted merge.
- An unchanged-order merge stays byte-identical to today's output.
