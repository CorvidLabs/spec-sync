---
id: CHG-0147-an-explicit-enforcement-policy-must-survive-migrate
state: archived
type: bug_fix
base_commit: 2db49c6cc4ea38aa13e0d40f4e19245bb9d41435
---

# An explicit enforcement policy must survive migrate

## Intent

an explicit enforcement policy must survive migrate

## Affected Canonical Specs

- `config`

## Acceptance Criteria

- a project with an explicit enforcement value has the same effective policy before and after migrate, demonstrated by check exiting identically on a tree with a validation error; the migrated config states the enforcement explicitly rather than relying on whichever default the binary carries; a project that never set enforcement is unaffected; the documented default matches the code

## No-spec Rationale

config_to_toml omitted enforcement when the value was Warn with the comment default-omit, but the default moved to Strict, so migrate dropped an explicit warn and the project silently became gating
