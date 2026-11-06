---
id: make-the-ship-status-product-stage-advertise-only-a-command-its-state-accepts
state: draft
type: bug_fix
base_commit: cd8cab5e1789246f6f3a4bf6733b33d301d3ca4b
---

# Make the ship-status product stage advertise only a command its state accepts

## Intent

Make the ship-status product stage advertise only a command its state accepts

## Affected Canonical Specs

- `cmd_change`

## Acceptance Criteria

- For an `accepted` or `archived` change whose verification is no longer current for this content, the `ship-status` product-tip stage action names `specsync change reopen <id> --actor <name> --reason <text>` instead of `change check --commit`, which refuses those states.
- For a `draft` change the stage action names `specsync change approve <id> --actor <name>` instead of `change check --commit`; approved, implementing and verifying changes keep `change check <id> --commit`.
- A unit test drives every state through the product stage with stale verification and asserts the advertised `specsync change` verb is one whose state guard admits that state. It fails on main first (draft and accepted advertise `check`).
- The top-level `ship_next`, readiness, stage ids and every gate are unchanged.

## No-spec Rationale

Not applicable
