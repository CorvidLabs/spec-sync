---
id: keep-lifecycle-tests-approvable-after-a-plan-must-name-its-sections
state: archived
type: bug_fix
base_commit: 09cf77f298753eb671e9384efd7952ad5c64e0be
---

# Keep lifecycle tests approvable after a plan must name its sections

## Intent

Keep lifecycle tests approvable after a plan must name its sections

## Affected Canonical Specs

- None

## Acceptance Criteria

- Lifecycle tests that select a plan still approve when that plan names its sections.

## No-spec Rationale

These tests only fill a selected plan so approval still succeeds. They do not change a contract.
