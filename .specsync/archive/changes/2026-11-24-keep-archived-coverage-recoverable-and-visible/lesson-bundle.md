# Lesson bundle: keep-archived-coverage-recoverable-and-visible

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Keep archived coverage recoverable and visible
- **Kind**: BugFix
- **Specs**: change, cmd_change
- **Paths**: src/change/historical_definition_digest_matches.rs, src/change/sdd_version.rs, src/change/collect_requirement_ids.rs, src/commands/change.rs, src/change_tests/provisional_coverage.rs, specs/change/, specs/cmd_change/
- **Acceptance**: An archived change with a generated coverage receipt can reopen and re-finalize using current coverage. A refused or interrupted reopen preserves the archived package and unexpected files. Status, show and audit --archives report a present invalid coverage receipt, while legitimately absent receipts remain silent and authenticated archive evidence stays valid.

## Evidence

- Verification commit: `4bd95ef9ce000589dcde0b06644b19f705df09f0`
- Base commit: `1abf8b806e1f6306c522710abfc8f975576b7941`
- Verified by: `specsync check --spec change --spec cmd_change`

## From the change's context.md

# Context

Two isolated command-line reproductions on the #1132 integration at `1abf8b80` show that a successful archived reopen strands the previous generation’s valid `coverage.json` in the active package, and that three read commands silently treat corrupt coverage as absent. Git history still authenticates the archive in both cases. The proposed repair is a distinct scope from #1132’s explicit succession work.

## From the change's design.md

# Design

Treat `coverage.json` as a generated receipt for one archive generation. A reopen from Archived may retire a bounded, structurally valid receipt belonging to that archive in the same durable file publication as the updated state, summary and reopen ledger. The journal must preserve the original bytes and recover them on write failure. Keep the existing confined path checks and archive directory rollback. A reopen from Accepted, an unexpected active copy, a corrupt receipt, or a receipt whose bytes change during the operation is never eligible for automatic cleanup. Restore or move aside unexpected data explicitly. Re-finalization runs the existing closing validation and writes only the new generation’s receipt, if any.

Use one bounded domain reader that distinguishes absence, valid coverage, and present invalid coverage. Check version, change ID, entry structure and canonical paths before looking up covering changes. Report the bad path and a safe restore instruction as an additive diagnostic. Keep the successful coverage rows and existing absent-file output shape. Archive authentication never consults this optional receipt.

## From the change's testing.md

# Testing

Extend the existing real lifecycle fixtures. First archive a provisionally covered workflow-v2 change, commit it, then reopen, re-check, review and finalize. Check that the old receipt survives unchanged in Git history, the new archive is valid, and no old receipt leaks into a new generation with no temporary cover. Trigger a write failure after a journaled target and verify the original archived files and location can be restored. Verify an unexpected active receipt still blocks archive and remains untouched.

For the public CLI, exercise archived status/show and `audit --archives` in text and JSON. The cases are genuinely absent, valid with a withdrawn covering change, truncated JSON, wrong ID, unsupported schema, invalid entry, and oversized or unsafe input. Verify that the read commands make no project writes, that invalid input has a diagnostic, and that authenticated terminal status and the archive audit’s warning exit behavior remain intact.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-reopened-coverage-receipt-001 | `reopened_covered_archive_recomputes_coverage_at_finalization`, `reopen_rollback_restores_the_receipt_and_preserves_replacements`, and `reopened_archive_preserves_untracked_and_edited_valid_coverage` in `src/change_tests/provisional_coverage.rs` |
| REQ-cmd-change-archived-coverage-diagnostics-001 | `a_present_invalid_archive_coverage_receipt_is_reported_without_changing_authentication` and `an_archive_coverage_restore_uses_only_the_literal_receipt_path` in `src/change_tests/provisional_coverage.rs`, plus the text and JSON CLI regressions |

## Where these lessons go

- `specs/change/change.3md`
- `specs/cmd_change/cmd_change.3md`
