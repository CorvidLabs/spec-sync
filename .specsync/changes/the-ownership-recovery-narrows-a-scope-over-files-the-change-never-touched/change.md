---
id: the-ownership-recovery-narrows-a-scope-over-files-the-change-never-touched
state: verifying
type: bug_fix
base_commit: 46f55652649a72ddf5e1715938b504b96686566b
---

# The ownership recovery narrows a scope over files the change never touched

## Intent

the ownership recovery narrows a scope over files the change never touched

## Affected Canonical Specs

- `change`
- `cmd_change`

## Acceptance Criteria

- ship-status, ship preflight and the review refusal name an unowned production input the change never modified since its base commit with a recovery that narrows affected_paths, and keep the declare-the-owner recovery only for unowned inputs the change modified; without a base commit or git every input keeps the declare recovery

## No-spec Rationale

Not applicable
