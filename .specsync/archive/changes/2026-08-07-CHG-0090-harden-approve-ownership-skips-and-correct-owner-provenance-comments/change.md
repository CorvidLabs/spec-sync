---
id: CHG-0090-harden-approve-ownership-skips-and-correct-owner-provenance-comments
state: archived
type: bug_fix
base_commit: 5fdd245bd9f25c0366f0c52bcffe636087722e1b
---

# Harden approve ownership skips and correct-owner provenance comments

## Intent

harden approve ownership skips and correct-owner provenance comments

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- empty affected_specs without no_spec_change fails ownership validation at approve; never-closed correct-owner comments state weaker provenance not equivalence

## No-spec Rationale

Not applicable
