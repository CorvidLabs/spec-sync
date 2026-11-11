---
id: merge-driver-keeps-a-one-sided-requirement-reorder
state: archived
type: bug_fix
base_commit: 949ced6a4935688757ed82ea74819fdc6494691b
---

# Merge driver keeps a one-sided requirement reorder

## Intent

merge driver keeps a one-sided requirement reorder

## Affected Canonical Specs

- `merge`

## Acceptance Criteria

- a one-sided reorder of existing requirement blocks survives the merge in either direction; divergent reorders conflict

## No-spec Rationale

REQ-merge-006 already requires a clean merge to keep everything both sides wrote; the dropped reorder violated it
