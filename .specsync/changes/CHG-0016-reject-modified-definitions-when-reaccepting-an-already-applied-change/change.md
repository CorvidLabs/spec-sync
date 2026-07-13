---
id: CHG-0016-reject-modified-definitions-when-reaccepting-an-already-applied-change
state: implementing
type: bug_fix
base_commit: b714411df7a19e19918a0ea932182243eddf83fd
---

# Reject modified definitions when reaccepting an already-applied change

## Intent

Reject modified definitions when reaccepting an already-applied change

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- Reacceptance of an already-applied change rejects any definition digest that differs from the audited pre-reopen contract; unchanged definitions still reaccept without reapplying canonical deltas; CLI and unit regressions prove the fail-closed behavior

## No-spec Rationale

Not applicable
