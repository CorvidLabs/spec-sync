---
change: ship-status-readiness-must-ask-whether-the-scoped-review-is-current-and-say-so-when-it-cannot-tell
artifact: research
---

# Research

## Where the two commands diverged

- `src/commands/change.rs:854` — `let review_present = review_path.is_file();`
- `src/commands/change.rs:925` — the `ready_to_finalize` conjunction consuming it
- `src/change.rs:5654` — `scoped_review_is_current`, the predicate `finalize` uses
- `src/change.rs:5996` — `accept_change_with_gate`'s `require_scoped_review` call site
- `src/change.rs:6733` — `ChangeSummary.scoped_review_current`, the caller that got it right

## Can `scoped_review_is_current` distinguish stale from unavailable?

Before this change: **no.** It is a nine-term boolean conjunction returning `bool`; every term that
fails produces the same `false`. Its git half, `review_commit_is_current_checked`, did carry
distinct reasons in a `Result<(), String>`, but its only two callers
(`review_commit_is_current` and `record_scoped_review`) both called `.is_ok()` on it, so no reason
ever escaped the module.

The ingredients were nevertheless all present, which is why the fix is a classification rather than
a new check:

- the content terms (`contract_digest`, `execution_digest`, `workspace_digest`) are decidable from
  the tree with no history at all, and a mismatch is a genuine, actionable negative;
- the walk's own failures split cleanly into "it ran and caught a forbidden change" (one branch,
  the `strip_prefix` / `matches!` rejection) and "it could not run" (every other branch: HEAD
  unresolvable, the reviewed commit unreachable, enumeration failure, the descendant and parent
  bounds).

## Why content is decided before history

Both halves can fail at once. A review that is stale by content AND anchored to a rewritten commit
is stale for a reason the reader can act on today; answering "the walk was unavailable" about it
would be true and useless. #689 already settled that readiness is a content question, so content is
asked first and the walk's availability is consulted only once the content agrees.

## Measured behaviour of the walk after a squash

`git merge --squash` leaves `review.implementation_commit` either absent (fresh clone) or present
but not an ancestor of HEAD (branch retained). Both leave `implementation_commit..HEAD` without a
range to walk, so the classification treats non-ancestry as unavailable rather than as a violation.
The recovery is real and already proven by drill 008: re-running `change check --commit` and
`change review` re-anchors the review and `finalize` succeeds.
