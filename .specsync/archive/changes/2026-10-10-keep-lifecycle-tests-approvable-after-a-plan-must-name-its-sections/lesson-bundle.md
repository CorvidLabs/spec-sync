# Lesson bundle — keep-lifecycle-tests-approvable-after-a-plan-must-name-its-sections

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Keep lifecycle tests approvable after a plan must name its sections
- **Kind**: BugFix
- **Paths**: tests/integration/change.rs, tests/integration/comment.rs, tests/integration/helpers.rs
- **Acceptance**: Lifecycle tests that select a plan still approve when that plan names its sections.

## Evidence

- Verification commit: `e880a1b10abb073d8ba24ef89f7134bdedfd6b6a`
- Base commit: `09cf77f298753eb671e9384efd7952ad5c64e0be`
- Verified by: `specsync check (no spec in scope)`

## From the change's context.md

# Context

The plan gate now refuses a selected plan that does not name its sections. The lifecycle integration tests were still writing a one-line plan and then approving. Those edits landed in the pull request without being named on a change, so `change audit` failed after the earlier changes were archived.

This change claims those three test files and nothing else.

## From the change's testing.md

# Testing

The existing lifecycle integration tests approve a change whose selected plan names Approach, Out of scope, Steps, Risks, and Constraints consulted. No new test file is added. `change audit` on this pull request is the check that these paths are covered.

## Where these lessons go

This change declared no affected specs, so there is no module context to fold into.
