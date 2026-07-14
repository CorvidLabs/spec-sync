---
id: CHG-0027-preserve-accepted-evidence-across-valid-later-sequence-claims
state: archived
type: bug_fix
base_commit: c98d29810f78abcdd6a2fec9b137667d3ab2fc5b
---

# Preserve accepted evidence across valid later sequence claims

## Intent

Preserve accepted evidence across valid later sequence claims

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- Creating a valid later change leaves prior accepted evidence current; manual sequence-ledger tampering and invalid owner claims still fail closed; focused regression tests and the full strict Trust lane pass.

## No-spec Rationale

Not applicable
