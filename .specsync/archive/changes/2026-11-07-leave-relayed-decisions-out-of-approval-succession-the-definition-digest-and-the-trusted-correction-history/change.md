---
id: leave-relayed-decisions-out-of-approval-succession-the-definition-digest-and-the-trusted-correction-history
state: archived
type: bug_fix
base_commit: 71130737616cf0379694f5e7b0446bceff4628e7
---

# Leave relayed decisions out of approval succession, the definition digest and the trusted correction history

## Intent

Leave relayed decisions out of approval succession, the definition digest and the trusted correction history

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- One of those records never changes what an approval, review or check decides, so an older SpecSync that drops it can still reopen and finish the change.

## No-spec Rationale

Not applicable
