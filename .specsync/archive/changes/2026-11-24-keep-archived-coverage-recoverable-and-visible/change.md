---
id: keep-archived-coverage-recoverable-and-visible
state: archived
type: bug_fix
base_commit: 1abf8b806e1f6306c522710abfc8f975576b7941
---

# Keep archived coverage recoverable and visible

## Intent

Keep archived coverage recoverable and visible

## Affected Canonical Specs

- `change`
- `cmd_change`

## Acceptance Criteria

- An archived change with a generated coverage receipt can reopen and re-finalize using current coverage. A refused or interrupted reopen preserves the archived package and unexpected files. Status, show and audit --archives report a present invalid coverage receipt, while legitimately absent receipts remain silent and authenticated archive evidence stays valid.

## No-spec Rationale

Not applicable
