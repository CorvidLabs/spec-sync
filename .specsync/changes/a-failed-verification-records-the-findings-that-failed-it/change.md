---
id: a-failed-verification-records-the-findings-that-failed-it
state: draft
type: bug_fix
base_commit: 46f55652649a72ddf5e1715938b504b96686566b
---

# A failed verification records the findings that failed it

## Intent

a failed verification records the findings that failed it

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- A failed verification command records a bounded output of the findings that failed it (errors, and warnings only under --strict), and a gate refusal records its refusal texts, so a recorded failure is classifiable without re-running it; passing records are byte-identical to before; the 4 KiB tail never splits a character and keeps the error that failed the check

## No-spec Rationale

Not applicable
