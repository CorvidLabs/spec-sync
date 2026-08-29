---
change: ship-status-readiness-must-ask-whether-the-scoped-review-is-current-and-say-so-when-it-cannot-tell
artifact: requirements
---

# Requirements

## `REQ-cmd-change-007` (MODIFIED)

`ship-status` reports scoped-review currency, treats only `current` as satisfied, blocks on a
decided stale review while warning on an undeterminable one, and never reports a change ready that
finalization will refuse on review currency.

## `REQ-change-046` (MODIFIED)

Scoped-review currency is a three-valued answer — `current`, `stale`, `unavailable` — and content is
decided before history.

## Out of scope

Whether an unavailable descendant guarantee should block finalization at all (#694). This change
makes the answer sayable and stops one caller reporting it as satisfied; it does not choose among
#694's three options, and the tests are written so that any of the three clears them.
