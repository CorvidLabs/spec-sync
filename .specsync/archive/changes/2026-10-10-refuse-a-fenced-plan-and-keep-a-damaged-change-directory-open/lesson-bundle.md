# Lesson bundle — refuse-a-fenced-plan-and-keep-a-damaged-change-directory-open

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Refuse a fenced plan and keep a damaged change directory open
- **Kind**: BugFix
- **Specs**: change, cmd_change
- **Paths**: .github/scripts/archive-readiness.sh, .github/scripts/test-archive-readiness.sh, specs/change/change.spec.md, specs/change/context.md, specs/change/requirements.md, specs/change/tasks.md, specs/change/testing.md, specs/cmd_change/cmd_change.spec.md, specs/cmd_change/context.md, specs/cmd_change/requirements.md, specs/cmd_change/tasks.md, src/change.rs, src/change_tests.rs
- **Acceptance**: A plan cannot be completed from inside a code fence, and a change directory that holds a file but no state.json stays open.

## Evidence

- Verification commit: `3b4262a2532bac958de636acabeb88e9fe2a7589`
- Base commit: `d2448bbbfdabfe37e44aaaceb361d03c762af6c2`
- Verified by: `specsync check --spec change --spec cmd_change`

## From the change's context.md

# Context

A fenced example in a plan was being read as the plan. A change directory that still held files but had lost state.json was being treated as finished. Both are closed. An empty directory husk is still skipped.

## From the change's testing.md

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-050 | `active_directory_with_files_and_no_state_is_still_open` passes, and an empty deltas husk is still skipped. |
| REQ-change-068 | The same test reports the missing state file and keeps sibling enumeration. |
| REQ-change-105 | `a_fenced_plan_cannot_be_completed_from_inside_a_code_block` passes, and the existing plan-section tests still pass. |
| REQ-cmd-change-019 | `.github/scripts/test-archive-readiness.sh` names `broken-notes` as still open and exits 0 for an empty husk. |

- `cargo test --bin specsync -- fenced_plan active_directory_with_files_and_no_state_is_still_open` passed.
- `bash .github/scripts/test-archive-readiness.sh` passed.

## Where these lessons go

- `specs/change/context.md`
- `specs/cmd_change/context.md`
