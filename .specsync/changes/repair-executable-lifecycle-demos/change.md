---
id: repair-executable-lifecycle-demos
state: implementing
type: bug_fix
base_commit: ffba9a32b664a7b3308350c162f0ac808f24efe3
---

# Repair executable lifecycle demos

## Intent

Repair executable lifecycle demos

## Affected Canonical Specs

- None

## Acceptance Criteria

- All three published demos exit successfully with the SpecSync 6 binary; generated slug IDs and selected artifacts drive lifecycle commands; dependent-before-prerequisite remains rejected; each archive is committed sequentially; the five-epic demo executes its product tests and cannot report success after test failure; CI executes the examples and negative controls

## No-spec Rationale

Repair standalone example scripts, their documentation, and executable example checks; no library or CLI runtime contract changes
