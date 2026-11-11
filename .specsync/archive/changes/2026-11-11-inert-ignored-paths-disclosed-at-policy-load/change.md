---
id: inert-ignored-paths-disclosed-at-policy-load
state: archived
type: bug_fix
base_commit: 46f55652649a72ddf5e1715938b504b96686566b
---

# Inert ignored paths disclosed at policy load

## Intent

inert ignored paths disclosed at policy load

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- a policy whose ignored_paths lists the specs tree or a protected policy file prints a load-time note naming the inert entry and its removal; adopt no longer writes the inert specs/ entry

## No-spec Rationale

Not applicable
