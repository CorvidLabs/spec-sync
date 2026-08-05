---
id: CHG-0086-return-src-commands-change-rs-to-its-sole-canonical-owner
state: implementing
type: bug_fix
base_commit: 8657252d962340931fe27a82fe4adb4b4f0c88e1
---

# Return src/commands/change.rs to its sole canonical owner

## Intent

Return src/commands/change.rs to its sole canonical owner

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- cmd_change is the sole canonical owner of src/commands/change.rs and no source file is claimed by two specs

## No-spec Rationale

Not applicable
