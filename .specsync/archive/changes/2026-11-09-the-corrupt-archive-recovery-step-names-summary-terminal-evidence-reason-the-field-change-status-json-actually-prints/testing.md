---
change: the-corrupt-archive-recovery-step-names-summary-terminal-evidence-reason-the-field-change-status-json-actually-prints
artifact: testing
---

# Testing

- Integration, through the built binary: `a_corrupt_archive_is_not_safe_to_leave_and_names_the_restore_that_recovers_it` now ends by folding the tamper into the commit that archived the package, so the package matches that commit. It asserts that the step reads the status JSON, then splits the backticked field path on `.` and walks it in the parsed `change status --json` output, requiring a non-empty string. Before the fix it fails: `terminal_evidence.reason` is null at the top level.
- Unit: the three expectations of the `OutsidePackage` step text (`corrupt_archive_restore_names_the_package_only_when_the_package_changed`, the `classify_handoff` branch test and the post-move-failure test) name `summary.terminal_evidence.reason`.
