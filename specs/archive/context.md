---
spec: archive.spec.md
---

## Key Decisions

- Archiving is line-based on `tasks.md` companions: completed items (`- [x]`/`- [X]`) are moved to an `## Archive` section appended at the bottom; everything else keeps its order.
- An existing `## Archive` section is parsed and preserved — its prior entries are re-emitted before the newly archived ones, so history accumulates rather than being overwritten.
- `archive_tasks` returns one typed `ArchiveReport`; planned, succeeded, rolled-back, and failed operations never have to be reconstructed from terminal text.
- Planning reads and validates every candidate before staging begins. Any planning failure prevents all staging and destination writes.
- Every replacement is staged in the destination directory with the original permissions before publication begins. `NamedTempFile::persist` performs the platform-specific atomic replacement.
- A late publication failure stops the transaction and attempts reverse-order rollback. Files that cannot be restored remain in `succeeded`, making `partial` state observable to the command.
- `dry_run` returns the same plan as apply without staging or writing.

## Files to Read First

- `src/archive.rs` — `archive_tasks`, the `archive_completed_tasks` core, and `count_completed_tasks`
- `src/validator.rs` — `find_spec_files` (how spec dirs and their companion `tasks.md` are discovered)
- `src/commands/archive_tasks.rs` — the `archive-tasks` subcommand wiring

## Current Status

Stable with transactional multi-file behavior. Unit coverage includes plan/stage/publish failure boundaries, rollback, permission preservation, and dry-run/apply parity; CLI integration covers clean previews and fail-closed structured output.
