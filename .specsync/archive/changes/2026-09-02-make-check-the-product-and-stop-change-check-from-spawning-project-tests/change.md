---
id: make-check-the-product-and-stop-change-check-from-spawning-project-tests
state: archived
type: feature
base_commit: 359eeee2981f72ce915a65ccc36ade84127d93a9
---

# Make check the product and stop change check from spawning project tests

## Intent

Make check the product and stop change check from spawning project tests

## Affected Canonical Specs

- `change`
- `cmd_change`
- `cmd_check`
- `cmd_init`
- `agents`

## Acceptance Criteria

- Fresh init writes SDD off and does not start a first-change interview.
- specsync check does not call audit_project or print an active-change count.
- change check compares specs to code in-process and does not spawn sdd.json verification_commands.
- A configured verification_commands sentinel is not executed.
- A phantom export still fails change check.
- change audit no longer re-runs project test commands in CI.

## No-spec Rationale

Not applicable
