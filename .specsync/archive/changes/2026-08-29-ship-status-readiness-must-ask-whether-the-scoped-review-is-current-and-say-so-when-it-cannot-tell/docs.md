---
change: ship-status-readiness-must-ask-whether-the-scoped-review-is-current-and-say-so-when-it-cannot-tell
artifact: docs
---

# Docs

No end-user documentation page changes. The behaviour is described where it is contracted:

- `specs/cmd_change/requirements.md` REQ-cmd-change-007 gains `review_currency` in the JSON
  contract and the criteria for stale, unavailable, and ship-status/finalize agreement.
- `specs/cmd_change/cmd_change.spec.md` invariant 6 already said readiness decides from evidence
  CURRENCY; it now states that the rule covers the scoped review as well as the verification.
- `specs/change/requirements.md` REQ-change-046 gains the three-valued currency answer and the
  content-before-history ordering.
- `specs/change/change.spec.md` Public API documents `ScopedReviewCurrency`, its `reason`, and
  `recorded_scoped_review_currency`.

`ship-status` output changes in two reader-visible ways: `Review:` now reads `recorded (current)`,
`recorded (stale)`, `recorded (unavailable)`, `recorded (unreadable)`, or `missing`, and the JSON
carries `review_currency` and `review_currency_reason` beside the existing `review_present`.
