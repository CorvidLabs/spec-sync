---
change: ship-status-readiness-must-ask-whether-the-scoped-review-is-current-and-say-so-when-it-cannot-tell
artifact: plan
---

# Plan

1. Introduce `ScopedReviewCurrency` and `scoped_review_currency` in `src/change.rs`, redefining
   `scoped_review_is_current` on top of it so no caller's behaviour changes.
2. Replace `review_commit_is_current_checked` with `review_commit_currency` plus `review_commit_walk`,
   classifying each existing failure branch as a violation or an unavailability. Both former callers
   move to the surviving boolean wrapper.
3. Expose `recorded_scoped_review_currency` beside `recorded_verification_is_current`.
4. In `ship_status_report`, compute the currency, add `review_currency` / `review_currency_reason` to
   the JSON, make `ready_to_finalize` require `Current`, and render stale as a blocker and
   unavailable as a warning.
5. Route the same answer through the sibling surfaces inside the report: the `review_tip` stage and
   the text `Review:` line, both of which asked existence only.
6. Add the discriminator, the control, and relabel the existing #689 squash test as an agreement
   assertion.
7. Move the canonical spec text in `specs/change` and `specs/cmd_change`.
