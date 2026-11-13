---
id: ship-status-reports-an-archived-change-s-review-as-done-by-finalize-and-classifies-its-re-archive-tip-as-archive-only
state: approved
type: bug_fix
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# Ship-status reports an archived change's review as done by finalize and classifies its re-archive tip as archive-only

## Intent

ship-status reports an archived change's review as done by finalize and classifies its re-archive tip as archive-only

## Affected Canonical Specs

- `cmd_change`
- `change`

## Acceptance Criteria

- on an archived change whose archive authenticates, ship-status reports review_currency current with a finalize reason, a done review stage and no change review suggestion; an archive commit that also removes the archived change's own package files is archive_only, and another change's package beside it is product

## No-spec Rationale

REQ-cmd-change-007's tip class mirrors CI's archive-only lane, which counts the archived change's own package paths as archive work; SHIP-9.a names the one command when it is not time, never a review that refuses. Leif classed #1038 a bug fix on 2026-11-13: for an archived change, report the review as done by finalize and suggest nothing
