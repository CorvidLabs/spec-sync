---
change: ship-status-readiness-must-ask-whether-the-scoped-review-is-current-and-say-so-when-it-cannot-tell
artifact: design
---

# Design

## The seam

A three-valued `ScopedReviewCurrency` in `src/change.rs`:

- `Current` — every recorded binding still agrees with the tree.
- `Stale(reason)` — a decided negative: a bound digest moved, or the descendant walk ran and
  inspected a commit that changed a path outside this change's own lifecycle records.
- `Unavailable(reason)` — the question could not be answered at all.

`scoped_review_is_current` becomes `scoped_review_currency(..) == Current`, so `finalize` and
`accept` keep the exact predicate they had; there is one implementation, not two.
`review_commit_is_current_checked` becomes `review_commit_currency`, whose walk body is factored
into `review_commit_walk` returning `Ok(None)` (clean) / `Ok(Some(reason))` (violation) /
`Err(reason)` (could not run) — the three-way split the old `Result<(), String>` could not express.

`recorded_scoped_review_currency` is the public entry, the scoped-review twin of the existing
`recorded_verification_is_current`.

## Why `Unavailable` is a warning and `Stale` is a blocker

`Stale` is a decided negative with a reachable repair, so it renders exactly like the verification
half #689 fixed: a blocker naming what moved and the re-review that fixes it.

`Unavailable` renders as a warning plus `ready_to_finalize: false`. A blocker would assert that an
unobtainable guarantee ought to block, which is #694's open question; a warning says "readiness
cannot confirm this" and names the re-review that re-anchors it. Either way the two commands agree,
which is the property that must hold under all three of #694's options.

## Why the reader-facing labels live in the command layer

`ShipReviewStatus` in `src/commands/change.rs` extends the domain's three answers with `Missing`
and `Unreadable`, which are questions about the artifact rather than about its currency. Keeping
those two variants out of `ScopedReviewCurrency` is what stops the domain type from becoming a
presentation type.

## Rejected

**`&& scoped_review_current` on the conjunction.** Correct for the stale case and catastrophic for
the unavailable one: it turns every squash-merging repository permanently unfinalizable, which is
the #689 regression with the sign flipped. The control test exists to keep anyone from reaching for
it again.
