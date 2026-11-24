# Lesson bundle: keep-archived-foreign-change-directories-outside-adoption-validation-and-import

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Keep archived foreign change directories outside adoption validation and import
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/validate_loaded_change.rs, tests/integration/change.rs
- **Acceptance**: For both OpenSpec and Spec Kit, preview and import ignore a symlink inside the real top-level archive directory and reject a symlink in an active change. Preview and refusal leave the destination unchanged. Real adoption imports only active content.

## Evidence

- Verification commit: `03c8994d1290a0124c84dc213262925309e0b195`
- Base commit: `892df7f5e00e941edf6d00d9d8368080347b0281`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Issue #432 established that adoption previews validate the same source as an import. The shared import helper lives in `src/change/validate_loaded_change.rs`, outside the original change’s named paths. Both workflows inspect active changes and preserve the foreign archive directory in place.

## From the change's testing.md

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-014 | `tests/integration/change.rs::adopt_dry_run_and_import_skip_archive_descendants_but_refuse_active_symlinks` runs SpecSync in disposable OpenSpec and Spec Kit projects, validates both modes and checks the destination contents. |

## Where these lessons go

- `specs/change/change.3md`
