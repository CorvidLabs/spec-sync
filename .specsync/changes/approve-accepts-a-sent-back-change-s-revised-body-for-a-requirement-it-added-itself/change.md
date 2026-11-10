---
id: approve-accepts-a-sent-back-change-s-revised-body-for-a-requirement-it-added-itself
state: implementing
type: bug_fix
base_commit: 82c724703d5f6b89fe0a68dc672d4b1dbd8e41dc
---

# Approve accepts a sent-back change's revised body for a requirement it added itself

## Intent

approve accepts a sent-back change's revised body for a requirement it added itself

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- after a send-back, change approve accepts a revised body for an ADDED requirement whose key the send-back recorded as this change's own applied item while the canonical text is still that item's text, and materialization then replaces the block; an ADDED requirement whose canonical text someone else changed is still refused with 'cannot add existing block'

## No-spec Rationale

REQ-change-123 already requires that after a send-back a kept ADDED item with a new body replaces the change's earlier body; materialization did, but approve refused the revised requirement body first. This change makes approve match the existing requirement
