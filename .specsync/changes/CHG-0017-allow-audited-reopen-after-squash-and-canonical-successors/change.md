---
id: CHG-0017-allow-audited-reopen-after-squash-and-canonical-successors
state: accepted
type: bug_fix
base_commit: ca766a0beca901b96978520afb07449dd1bd89e7
---

# Allow audited reopen after squash and canonical successors

## Intent

Allow audited reopen after squash and canonical successors

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- Reopen accepts passed accepted evidence with unchanged definition and valid closing approval when canonical acceptance is recorded in current history or every governed contract surface has a later recorded canonical successor; stale inputs and an explicit audit reason remain required; arbitrary off-history evidence remains rejected; squash and successor regressions pass

## No-spec Rationale

Not applicable
