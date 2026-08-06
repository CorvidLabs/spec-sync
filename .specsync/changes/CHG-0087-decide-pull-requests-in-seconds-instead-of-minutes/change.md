---
id: CHG-0087-decide-pull-requests-in-seconds-instead-of-minutes
state: approved
type: bug_fix
base_commit: e847ab19f66a7f8720a63ed9c19fa496087a0bff
---

# Decide pull requests in seconds instead of minutes

## Intent

Decide pull requests in seconds instead of minutes

## Affected Canonical Specs

- `github`

## Acceptance Criteria

- a pull request whose active change has orphaned verification evidence fails in seconds without a build, and the expensive jobs do not run

## No-spec Rationale

Not applicable
