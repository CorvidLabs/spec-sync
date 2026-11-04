---
id: send-an-approved-change-back-to-draft-for-re-scoping
state: approved
type: feature
base_commit: f82eb09e2d3da852ff58e0cfbb734ab6faba724e
---

# Send an approved change back to draft for re-scoping

## Intent

Send an approved change back to draft for re-scoping

## Affected Canonical Specs

- `change`
- `cmd_change`
- `cli_args`

## Acceptance Criteria

- When a change's scope turns out wrong after approval, I can send it back to be re-scoped instead of deleting it by hand.
- Sending it back needs a named person and a reason, and both stay in the change's history.
- Sending it back cancels its approval, so the new scope needs a fresh one.
- Nothing already recorded about the change is lost.

## No-spec Rationale

Not applicable
