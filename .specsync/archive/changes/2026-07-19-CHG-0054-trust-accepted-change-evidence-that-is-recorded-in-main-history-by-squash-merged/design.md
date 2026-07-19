---
change: CHG-0054-trust-accepted-change-evidence-that-is-recorded-in-main-history-by-squash-merged
artifact: design
---

# Design

Keep every existing authentication check and widen only which commits qualify as accepted-evidence
anchors. When the first-acceptance transition search (and, for archived records, the
`accepted-state.json` addition scan) finds no eligible anchor, fall back to a recording-anchor
search: every commit reachable from `HEAD` or the remote default whose `state.json` records this
change as `accepted`.

Each recording anchor must pass the identical per-anchor filters as a transition anchor: the
committed verification and approvals bytes must equal the current workspace evidence, the committed
state must parse as this change in `accepted`, and the current record must project exactly onto the
committed accepted snapshot. Matching anchors deduplicate through the existing evidence key, and
the exactly-one-eligible rule still fails closed on zero or ambiguous results.

This is sound because a commit on `main` that atomically records `(accepted, verification,
approvals)` binds the same three artifacts a transition commit does; the closing-approval digest
check elsewhere cryptographically binds the approval to the verification, so the parent-state
requirement adds no authentication strength. The fallback only runs when the established searches
find nothing, so behavior for merge-commit and first-acceptance histories is unchanged.
