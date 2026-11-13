---
id: ship-status-names-change-reopen-for-an-accepted-or-archived-change-only-when-its-own-accepted-evidence-is-stale-and
state: draft
type: bug_fix
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# Ship-status names change reopen for an accepted or archived change only when its own accepted evidence is stale and readable

## Intent

ship-status names change reopen for an accepted or archived change only when its own accepted evidence is stale and readable

## Affected Canonical Specs

- `cmd_change`
- `change`

## Acceptance Criteria

- an archived change whose own accepted evidence is current is not told to reopen after an unrelated committed edit, and its product stage is done; an archived change whose verification.json cannot be read is told to restore it, not to reopen; an accepted or archived change with stale accepted evidence is still told to reopen

## No-spec Rationale

REQ-cmd-change-033 lets the product-tip stage advertise only a command whose state guard admits the change; change reopen refuses accepted evidence that is current (unrelated project drift) and evidence it cannot load, so naming it there advertised a refused command (#922)
