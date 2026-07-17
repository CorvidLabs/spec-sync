---
id: CHG-0053-permit-audited-reopened-collision-members-to-retain-immutable-sequence-history-s
state: implementing
type: bug_fix
base_commit: 5590b2cb1fc2328c5141472a47e852a7695ed0ca
---

# Permit audited reopened collision members to retain immutable sequence-history status during re-verification

## Intent

Permit audited reopened collision members to retain immutable sequence-history status during re-verification

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- An acknowledged accepted collision member can be audited-reopened and reaccepted without invalidating the collision acknowledgement
- Only an already-applied verifying record with structurally valid reopen evidence retains historical collision status
- Both CHG-0048 records are reopened against the integrated Action documentation, fully reverified, and reaccepted

## No-spec Rationale

Not applicable
