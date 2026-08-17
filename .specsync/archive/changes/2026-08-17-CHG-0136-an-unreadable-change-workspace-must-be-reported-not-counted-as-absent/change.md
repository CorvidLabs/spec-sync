---
id: CHG-0136-an-unreadable-change-workspace-must-be-reported-not-counted-as-absent
state: archived
type: bug_fix
base_commit: bb6b5ee9a8a4dbf5bf4e5c50762e0855b796e718
---

# An unreadable change workspace must be reported, not counted as absent

## Intent

an unreadable change workspace must be reported, not counted as absent

## Affected Canonical Specs

- `change`
- `cmd_change`

## Acceptance Criteria

- change list and change status print every readable change, name each unreadable workspace with its reason including the offending path, and exit non-zero; a project with genuinely no active changes still prints the empty-project line and exits 0; JSON keeps its bare-array shape when every workspace is readable and becomes an object carrying changes plus unreadable when degraded, always one parseable document; ship and lifecycle commit resolution refuse to infer a target while any workspace is unreadable; sibling-active reporting counts unreadable workspaces as active; sandbox gate 055 goes pass=5 pending=2 to pass=7 pending=0 with all five controls still green; the whole board moves 43/12 to 44/11 with exactly one drill changing state.

## No-spec Rationale

Not applicable
