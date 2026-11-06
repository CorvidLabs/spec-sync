---
id: keep-the-upstream-archive-status-guidance-behind-a-sequence-ledger-freeze
state: draft
type: bug_fix
base_commit: cd8cab5e1789246f6f3a4bf6733b33d301d3ca4b
---

# Keep the upstream-archive status guidance behind a sequence-ledger freeze

## Intent

Keep the upstream-archive status guidance behind a sequence-ledger freeze

## Affected Canonical Specs

- `change`
- `cmd_change`

## Acceptance Criteria

- When a change is archived upstream and the change-sequence ledger is frozen, `change status --json` gives the freeze in `next_action` and in `handoff.before_clearing`, not the upstream-archive rebase step.
- Text `change status` and the ship-status lifecycle next action give the same freeze step in that case.
- Without a freeze, the upstream-archive guidance is unchanged: text, JSON next_action and handoff all name it.
- An integration test built from the auditor's squash fixture plus a frozen ledger fails on main first and passes after.

## No-spec Rationale

Not applicable
