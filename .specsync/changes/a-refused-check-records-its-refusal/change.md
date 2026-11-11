---
id: a-refused-check-records-its-refusal
state: implementing
type: bug_fix
base_commit: 46f55652649a72ddf5e1715938b504b96686566b
---

# A refused check records its refusal

## Intent

a refused check records its refusal

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- a change check refused before verification appends a passed:false attempt with the reason to verification-attempts.json; verification.json is untouched

## No-spec Rationale

Not applicable
