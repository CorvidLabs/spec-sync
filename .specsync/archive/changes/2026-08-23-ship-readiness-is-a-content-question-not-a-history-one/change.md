---
id: ship-readiness-is-a-content-question-not-a-history-one
state: archived
type: bug_fix
base_commit: db36230a1b56717ec92501239a3219dbf9b58219
---

# Ship readiness is a content question, not a history one

## Intent

ship readiness is a content question, not a history one

## Affected Canonical Specs

- `change`
- `cmd_change`

## Acceptance Criteria

- A change whose verification commit was destroyed by a squash-merge, with its content unchanged, reports ready_to_finalize with no blockers. Evidence whose workspace digest does not match the tree still reads as stale. The discriminating test fails on a binary built from a separate checkout at db36230a with ready_to_finalize false and the ancestry blocker present.

## No-spec Rationale

Not applicable
