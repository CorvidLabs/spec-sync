---
id: let-a-change-reviewed-before-its-send-back-be-reviewed-again-after-re-approval
state: archived
type: bug_fix
base_commit: cd8cab5e1789246f6f3a4bf6733b33d301d3ca4b
---

# Let a change reviewed before its send-back be reviewed again after re-approval

## Intent

Let a change reviewed before its send-back be reviewed again after re-approval

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- A change that was reviewed, sent back, re-answered, re-approved and re-checked can be reviewed again: `change review` records the new review instead of refusing with `scoped review does not bind a recorded definition approval`, and `change finalize` archives it.
- The review recorded before the send-back stays in the append-only review history and in committed history, and still validates against the approval it bound, including an approval the send-back cancelled; it is never removed or rewritten.
- Until the fresh review, `change status` and `ship-status` report the earlier review stale (not unreadable) and name `change review`, and `change ship` refuses.
- Whether a review counts is decided as before: it must match the current contract and evidence, and a review bound only to an approval a send-back cancelled is never current (CHANGE-12.b, REQ-change-123); no gate, bound or timeout is relaxed.
- The integration test built from the #914 repro fails on main cd8cab5e and passes with the fix.

## No-spec Rationale

Not applicable
