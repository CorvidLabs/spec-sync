---
spec: archive.spec.md
---

## Key Decisions

- Archiving is line-based on `tasks.md` companions: completed items (`- [x]`/`- [X]`) are moved to an `## Archive` section appended at the bottom; everything else keeps its order.
- An existing `## Archive` section is parsed and preserved — its prior entries are re-emitted before the newly archived ones, so history accumulates rather than being overwritten.
- Errors are non-fatal: an unreadable/unwritable `tasks.md` prints a red error and processing continues with the next file.
- `dry_run` computes results without writing, so callers can preview exactly which files would change.

## Files to Read First

- `src/archive.rs` — `archive_tasks`, the `archive_completed_tasks` core, and `count_completed_tasks`
- `src/validator.rs` — `find_spec_files` (how spec dirs and their companion `tasks.md` are discovered)
- `src/commands/archive_tasks.rs` — the `archive-tasks` subcommand wiring

## Current Status

Stable and complete. Core logic is unit-tested in `src/archive.rs` (archive, no-completed, preserve-existing). No CLI-level integration test yet.
