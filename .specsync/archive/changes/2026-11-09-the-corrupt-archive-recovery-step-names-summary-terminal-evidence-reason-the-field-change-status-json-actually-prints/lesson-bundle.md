# Lesson bundle: the-corrupt-archive-recovery-step-names-summary-terminal-evidence-reason-the-field-change-status-json-actually-prints

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: The corrupt-archive recovery step names summary.terminal_evidence.reason, the field change status --json actually prints (#966)
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change, src/change_tests, tests/integration
- **Acceptance**: When a corrupt archive's package still matches the commit that archived it, the recovery step change status prints names summary.terminal_evidence.reason, and reading that exact path from change status --json gives a non-empty reason (#966, CHANGE-7.a.2)

## Evidence

- Verification commit: `43756b1bf8f06502f02ff9353c1adaa3a59f501f`
- Base commit: `75df08756cafeb096dbf03a6208db17c8c8a31fa`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Issue #966, from the Codex review of #901 (P2, "Point recovery guidance at the emitted JSON field"), deferred by Leif's call.

When an archive fails its integrity check but its package still matches the commit that archived it (`CorruptArchive::OutsidePackage`), the recovery step says to run `specsync change status '<id>' --json` and restore what its `terminal_evidence.reason` names. `change status --json` nests the change summary under `summary` (`print_record`), so that top-level field does not exist and the step cannot be followed as written.

HI: CHANGE-7.a.2, "I am told the exact command that restores that archive from trusted history, and once it is restored I am told it is safe again."

## From the change's testing.md

# Testing

- Integration, through the built binary: `a_corrupt_archive_is_not_safe_to_leave_and_names_the_restore_that_recovers_it` now ends by folding the tamper into the commit that archived the package, so the package matches that commit. It asserts that the step reads the status JSON, then splits the backticked field path on `.` and walks it in the parsed `change status --json` output, requiring a non-empty string. Before the fix it fails: `terminal_evidence.reason` is null at the top level.
- Unit: the three expectations of the `OutsidePackage` step text (`corrupt_archive_restore_names_the_package_only_when_the_package_changed`, the `classify_handoff` branch test and the post-move-failure test) name `summary.terminal_evidence.reason`.

## Where these lessons go

- `specs/change/change.3md`
