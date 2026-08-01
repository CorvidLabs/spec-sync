---
id: CHG-0073-approve-rejects-living-added-reqs-and-draft-next-action-waits-on-complete-artifa
state: implementing
type: bug_fix
base_commit: 109164ad9faccb598ae7e8caf7a2d488722cc237
---

# Approve rejects living ADDED REQs and draft next_action waits on complete artifacts

## Intent

Approve rejects living ADDED REQs and draft next_action waits on complete artifacts

## Affected Canonical Specs

- `change`
- `cmd_change`

## Acceptance Criteria

- Approve fails closed when ## ADDED targets a living requirement ID and steers agents to ## MODIFIED; draft next_action prefers completing incomplete selected artifacts over approve when artifacts_complete is false.

## No-spec Rationale

Not applicable
