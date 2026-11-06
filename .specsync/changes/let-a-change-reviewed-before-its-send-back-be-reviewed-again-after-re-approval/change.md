---
id: let-a-change-reviewed-before-its-send-back-be-reviewed-again-after-re-approval
state: draft
type: bug_fix
base_commit: cd8cab5e1789246f6f3a4bf6733b33d301d3ca4b
---

# Let a change reviewed before its send-back be reviewed again after re-approval

## Intent

Let a change reviewed before its send-back be reviewed again after re-approval

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- A change that was reviewed, sent back, re-answered, re-approved and re-checked can be reviewed again: `change review` records the new review instead of refusing with `scoped review does not bind a recorded definition approval`, and `change ship`/`finalize` archive it.
- The review recorded before the send-back stays in the append-only review history and in committed history, and still validates against the approval it bound; it is never removed or rewritten.
- A review recorded before the latest send-back never counts as current: `change status` and `ship-status` report it stale (not unreadable) with the re-review next action, and finalize refuses it, even when the re-approved definition is byte-identical.
- A current review must still bind a definition approval recorded after the latest send-back (CHANGE-12.b, REQ-change-123); no gate, bound or timeout is relaxed.
- The integration test built from the #914 repro fails on main cd8cab5e at the second review and passes with the fix.

## No-spec Rationale

Not applicable
