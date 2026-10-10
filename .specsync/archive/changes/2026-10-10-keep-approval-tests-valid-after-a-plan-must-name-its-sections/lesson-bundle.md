# Lesson bundle — keep-approval-tests-valid-after-a-plan-must-name-its-sections

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Keep approval tests valid after a plan must name its sections
- **Kind**: BugFix
- **Specs**: change
- **Paths**: specs/change/change.spec.md, specs/change/context.md, specs/change/requirements.md, specs/change/tasks.md, specs/change/testing.md, src/change_tests.rs
- **Acceptance**: The approval tests pass, and changing the principles file still invalidates that approval.

## Evidence

- Verification commit: `6ce5d8e95be2a322cbfe2b4d9adec3a00e527f05`
- Base commit: `81e0d1181f8983aee9630aea20d09304830bb787`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Two approval tests still treated a one-line plan as complete, and the principles test never answered the new constraints question. Both failed in CI after the plan gate and the constraints question landed. The tests now fill a real plan, and the principles test records `none` before approval.

## From the change's testing.md

# Testing

REQ-change-107

`project_principles_are_part_of_the_approval_digest` and `stale_definition_approval_is_rejected` both pass. Changing the principles file still invalidates the approval. Changing the approved context still invalidates a workflow-v1 approval.

## Where these lessons go

- `specs/change/context.md`
