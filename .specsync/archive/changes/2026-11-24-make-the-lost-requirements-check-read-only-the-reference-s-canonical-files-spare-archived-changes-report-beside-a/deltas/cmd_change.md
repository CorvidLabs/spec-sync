## ADDED

### REQUIREMENT REQ-cmd-change-045

`change ship-status` SHALL list the blocker of REQ-cmd-change-042 only for a change that can still be checked, finalized or shipped, and SHALL NOT list it for an archived or retired change (#1113).

Acceptance Criteria
- On a branch that lost a requirement after its change was archived, `change ship-status <archived-id>` lists no lost-requirements blocker.
- An active change on the same branch still lists it.
