---
id: answer-archived-change-status-change-list-and-change-status-within-2-seconds-on-raven
state: implementing
type: bug_fix
base_commit: 84dbb56b245da36b3625440f5e394d85d686cd22
---

# Answer archived change status, change list and change status within 2 seconds on Raven

## Intent

Answer archived change status, change list and change status within 2 seconds on Raven

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- Archived change status, change list and change status return in under 2 seconds on Raven, check stays proportional to the specs touched, read caches are keyed by content digest with a correct invalidation rule, and a benchmark against a Raven-sized fixture guards the budget; output is identical to a live read.

## No-spec Rationale

Not applicable
