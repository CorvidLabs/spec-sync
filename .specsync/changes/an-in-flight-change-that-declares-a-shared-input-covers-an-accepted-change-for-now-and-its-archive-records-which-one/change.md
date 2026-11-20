---
id: an-in-flight-change-that-declares-a-shared-input-covers-an-accepted-change-for-now-and-its-archive-records-which-one
state: implementing
type: feature
base_commit: a28e30420c12627a384a685cd5d808fc3585b70a
---

# An in-flight change that declares a shared input covers an accepted change for now, and its archive records which one

## Intent

an in-flight change that declares a shared input covers an accepted change for now, and its archive records which one

## Affected Canonical Specs

- `change`
- `cmd_change`

## Acceptance Criteria

- An accepted change whose signed input a later approved, implementing or verifying change has edited, and whose module and path that change declares, reads `provisionally-covered` naming it; `change audit` and `change audit --strict` exit 0 with a warning naming both changes; with no such change it is stale and audit fails as before.
- `change archive` accepts a provisionally covered change and writes `coverage.json` naming each covered input and its covering change; a provisionally covered change neither covers another change nor replaces one in a retirement, and `change reopen` still accepts it.
- An archived change that was covered for now reads `authenticated-history` whatever becomes of the covering change; when that change is sent back or closed, `change status`, `change show` and `change audit --archives` warn that the input has no accepted change behind it and name `change reopen`.
- Each of the two archive refusals over one stale delivery input names the other change and says it is blocked too, and the `change reopen` it names frees both.
- Released SpecSync 6.3.0 reads an archive that holds `coverage.json` as before: archived, `authenticated-history`, audit and doctor clean.

## No-spec Rationale

Not applicable
