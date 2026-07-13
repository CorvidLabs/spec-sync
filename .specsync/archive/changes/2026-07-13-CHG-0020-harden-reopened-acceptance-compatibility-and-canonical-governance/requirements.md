---
change: CHG-0020-harden-reopened-acceptance-compatibility-and-canonical-governance
artifact: requirements
---

# Requirements

## REQ-change-020

Audited reacceptance SHALL preserve compatible legacy definition evidence while requiring semantic successor governance and validation of every current canonical contract it reapproves.

Acceptance Criteria

- A prior verification digest using the transitional explicit-false lifecycle encoding remains compatible with the stable omitted-false encoding during reopened reacceptance.
- An accepted no-spec change cannot satisfy the canonical-successor fallback, even when its affected paths and specs overlap.
- A later recorded semantic canonical change can satisfy successor governance for every overlapping affected spec and path.
- A reopened canonical-applied change validates its current canonical modules without replaying its already-applied semantic delta.
