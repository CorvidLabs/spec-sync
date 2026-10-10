# Lesson bundle — close-a-fence-only-on-whitespace-and-count-a-symlink-as-a-file

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Close a fence only on whitespace and count a symlink as a file
- **Kind**: BugFix
- **Specs**: change, cmd_change
- **Paths**: .github/scripts/archive-readiness.sh, .github/scripts/test-archive-readiness.sh, .specsync/archive/changes/2026-10-10-refuse-a-fenced-plan-and-keep-a-damaged-change-directory-open/testing.md, hi/three.md, specs/change/change.spec.md, specs/change/requirements.md, specs/cmd_change/requirements.md, src/change.rs, src/change_tests.rs
- **Acceptance**: A code fence closes only when the marker is followed by whitespace, and a symlink keeps a state-less change directory open.

## Evidence

- Verification commit: `87d636372cdc757fd7af1b036622fcc2db673a2c`
- Base commit: `7713e417e406a53593381e7f934135fa95d163e1`
- Verified by: `specsync check --spec change --spec cmd_change`

## From the change's context.md

# Context

A closing fence with an info string was ending the code block early. A symlink-only directory was a husk in the Archive script and a damaged change in Rust. Both now agree.

## From the change's testing.md

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-050 | `active_directory_with_files_and_no_state_is_still_open` reports a symlink-only directory as unreadable. |
| REQ-change-068 | The same test keeps the healthy sibling and names the missing state file. |
| REQ-change-105 | `a_fenced_plan_cannot_be_completed_from_inside_a_code_block` refuses an info-string closer and accepts a bare longer closer. |
| REQ-cmd-change-019 | `.github/scripts/test-archive-readiness.sh` names `linked-notes` as still open. |

- `cargo test --bin specsync -- fenced_plan` passed.
- `cargo test --bin specsync -- active_directory_with_files_and_no_state_is_still_open` passed.
- `bash .github/scripts/test-archive-readiness.sh` passed.

## Where these lessons go

- `specs/change/context.md`
- `specs/cmd_change/context.md`
