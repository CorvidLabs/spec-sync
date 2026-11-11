---
id: merge-driver-keeps-a-one-sided-requirement-reorder
state: implementing
type: bug_fix
base_commit: 46f55652649a72ddf5e1715938b504b96686566b
---

# Merge driver keeps a one-sided requirement reorder

## Intent

merge driver keeps a one-sided requirement reorder

## Affected Canonical Specs

- `merge`

## Acceptance Criteria

- when one branch reorders existing requirement blocks and the other only adds or edits, the merge keeps the reorder; divergent reorders conflict

## No-spec Rationale

Not applicable
