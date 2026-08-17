---
id: CHG-0140-a-stale-sequence-ledger-must-not-be-committed-backwards
state: implementing
type: bug_fix
base_commit: 55c566d411612ce051678f6102066684de72b307
---

# A stale sequence ledger must not be committed backwards

## Intent

a stale sequence ledger must not be committed backwards

## Affected Canonical Specs

- `change`
- `cmd_change`

## Acceptance Criteria

- a lifecycle commit never records a sequence ledger below the one already committed at HEAD; the working-tree ledger is raised to the committed high-water mark before git add -A, at every one of the three lifecycle staging sites rather than only the one the report named; the raise is disclosed on stderr naming both values, so it survives --quiet and stays off a --format json payload; a working tree AHEAD of the committed mark is left untouched, because that is the ordinary case change new produces and raising it would destroy the author's claim; equal marks are not reported as a divergence; the author is never blocked, since the branch merely sat while main moved; sandbox drill 037 inverts from pinning the regression to pinning the repair and fails with the high-water diagnostic on an origin/main binary.

## No-spec Rationale

Not applicable
