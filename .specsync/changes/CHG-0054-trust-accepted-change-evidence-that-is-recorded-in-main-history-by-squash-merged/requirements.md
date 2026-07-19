---
change: CHG-0054-trust-accepted-change-evidence-that-is-recorded-in-main-history-by-squash-merged
artifact: requirements
---

# Requirements

Accepted-change archival SHALL trust squash-merged evidence when an in-history commit records the
change as accepted with byte-identical state, verification, and approvals.

- Only commits reachable from `HEAD` or the remote default qualify as recording anchors.
- Byte equality, accepted-state identity, and projection checks remain mandatory per anchor.
- The exactly-one-eligible rule still fails closed on missing or ambiguous evidence.
- First-acceptance transition anchors and the archived `accepted-state.json` scan keep priority;
  the fallback runs only when they find nothing.
- Changes with no matching in-history accepted record remain unarchivable.
