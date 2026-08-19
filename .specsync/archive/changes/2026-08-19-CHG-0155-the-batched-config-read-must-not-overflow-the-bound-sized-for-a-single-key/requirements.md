---
change: CHG-0155-the-batched-config-read-must-not-overflow-the-bound-sized-for-a-single-key
artifact: requirements
---

# Requirements

## REQ-change-077 (new)

A bounded Git read SHALL be bounded for the response it can actually receive, not for the
response the call it replaced received.

See `deltas/change.md` for the canonical delta.

## Deliberately unchanged

The deterministic-output guard itself, every value derived, and every error surfaced. This
change moves one constant and adds the test that should have accompanied CHG-0154.
