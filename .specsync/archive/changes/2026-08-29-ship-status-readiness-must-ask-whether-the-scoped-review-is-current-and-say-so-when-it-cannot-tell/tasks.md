---
change: ship-status-readiness-must-ask-whether-the-scoped-review-is-current-and-say-so-when-it-cannot-tell
artifact: tasks
---

# Tasks

- [x] Classify scoped-review currency three ways in `src/change.rs` without changing any existing
      caller's boolean result.
- [x] Split the descendant walk's failures into violations and unavailabilities.
- [x] Expose `recorded_scoped_review_currency`.
- [x] Make `ready_to_finalize` require a decided-current review.
- [x] Render stale as a blocker naming what moved; render unavailable as a warning that says the
      currency could not be determined.
- [x] Carry the same answer into the `review_tip` stage and the text `Review:` line.
- [x] Add the discriminator (stale by content) asserting agreement between ship-status and finalize.
- [x] Add the control (current review still reaches `ready_to_finalize: true`) with an honest label.
- [x] Relabel the #689 squash test as an agreement assertion.
- [x] Move the canonical spec text for both affected modules.
