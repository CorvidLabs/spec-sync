---
id: close-fenced-delta-replay-review-findings
state: archived
type: bug_fix
base_commit: 4df769d3130d17c39ead061077400b784bce2d13
---

# Close fenced delta replay review findings

## Intent

Close fenced delta replay review findings

## Affected Canonical Specs

- `change`
- `validator`

## Acceptance Criteria

- The existing requirement reader and the canonical materializer apply the same fence rules. A malformed backtick opener leaves both requirement and spec section headings visible. Valid fenced examples leave the intended block boundaries intact.
- Already-applied checks accept a removal when only a fenced example remains and a modification when the canonical block already carries the new body.

## No-spec Rationale

The materializer adopts the existing requirement reader fence rules and tests the existing replay contract. The reader behavior and normative requirements stay the same.
