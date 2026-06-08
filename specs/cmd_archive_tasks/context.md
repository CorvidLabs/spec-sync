---
spec: cmd_archive_tasks.spec.md
---

## Key Decisions

- Thin command wrapper: load config, resolve `specs_dir`, call `archive::archive_tasks`, format output. No domain logic lives here.
- Dry-run is passed straight through to the delegate; this module only flips the printed verb ("would archive" vs "archived") and prints the banner.
- Empty result is treated as a success case ("No completed tasks to archive.") with an early return — not an error.

## Files to Read First

- `src/commands/archive_tasks.rs` — the command wrapper (this module)
- `src/archive.rs` — `archive_tasks` + `ArchiveResult { tasks_path, archived_count }`, where the real parsing/rewriting lives
- `src/config.rs` — `load_config` / `specs_dir` resolution

## Current Status

Implemented and stable. The delegate `archive` module is well unit-tested; the wrapper itself has no inline tests (output formatting only).

## Notes

- `ArchiveResult.tasks_path` is already a repo-relative string produced by the delegate; the wrapper prints it verbatim.
