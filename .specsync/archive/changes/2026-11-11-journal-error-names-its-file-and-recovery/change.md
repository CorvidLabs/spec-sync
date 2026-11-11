---
id: journal-error-names-its-file-and-recovery
state: archived
type: bug_fix
base_commit: 46f55652649a72ddf5e1715938b504b96686566b
---

# Journal error names its file and recovery

## Intent

journal error names its file and recovery

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- a corrupt change-transaction.json error names the path and how to set it aside; every printed 'change reopen' remediation carries --actor and --reason

## No-spec Rationale

Not applicable
