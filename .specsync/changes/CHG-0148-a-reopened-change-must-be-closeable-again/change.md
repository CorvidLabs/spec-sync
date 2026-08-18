---
id: CHG-0148-a-reopened-change-must-be-closeable-again
state: implementing
type: bug_fix
base_commit: 34ade838f19840dfae90611e8959480c07b70f6b
---

# A reopened change must be closeable again

## Intent

a reopened change must be closeable again

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- a change that has been finalized, reopened, re-checked and re-reviewed can be finalized again, producing exactly one archive package and clearing its active workspace; a move of scoped review evidence to any location other than the change's two canonical homes is still refused; the ordinary first close is unchanged

## No-spec Rationale

the evidence walk admitted a move between a change's two canonical homes in one direction only, so reopen stranded the change in accepted with finalize refusing permanently and audit reporting passed over it
