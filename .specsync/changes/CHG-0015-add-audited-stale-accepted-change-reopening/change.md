---
id: CHG-0015-add-audited-stale-accepted-change-reopening
state: verifying
type: feature
base_commit: 59bbfa766c6cce01ab815ab47db195b0629cc014
---

# Add audited stale accepted change reopening

## Intent

Add audited stale accepted change reopening

## Affected Canonical Specs

- `change`
- `cmd_change`
- `cli_args`

## Acceptance Criteria

- Accepted stale evidence fails strict checks; an explicit human can reopen it with actor and reason; prior evidence remains inspectable; fresh verification and closing approval restore a strict pass; current evidence cannot be reopened; canonical deltas are not reapplied

## No-spec Rationale

Not applicable
