---
id: keep-approval-tests-valid-after-a-plan-must-name-its-sections
state: implementing
type: bug_fix
base_commit: 81e0d1181f8983aee9630aea20d09304830bb787
---

# Keep approval tests valid after a plan must name its sections

## Intent

Keep approval tests valid after a plan must name its sections

## Affected Canonical Specs

- None

## Acceptance Criteria

- The approval tests pass, and changing the principles file still invalidates that approval.

## No-spec Rationale

The tests only complete a selected plan and record that principles constrain the draft. They do not change a contract.
