---
spec: cmd_archive_tasks.spec.md
---

## Key Decisions

- Thin command wrapper: load config, resolve `specs_dir`, call `archive::archive_tasks`, render the typed report, and map incompleteness to exit 1. No task parsing or filesystem mutation logic lives here.
- Dry-run is passed straight through to the delegate; rendering uses the report plan while keeping `applied` false.
- Empty result is treated as a success case ("No completed tasks to archive.") with an early return — not an error.
- `OutputFormat::Json` produces one document with planned/succeeded/rolled-back/failed arrays plus `would_change`, `applied`, `complete`, and `partial`.
- `OutputFormat::Markdown` and `Github` share a PR-suitable renderer with one code element per path: dynamic-backtick spans normally and entity-safe HTML code for literal-pipe paths, with unchanged legal Unix backslashes.
- Paths stay as `PathBuf` values until rendering. Only Windows separators are normalized; a legal Unix backslash is preserved.
- Text rendering passes paths and filesystem errors through the same visible control/bidirectional escaping used by structured diagnostics.
- Failure reports are rendered and stdout is flushed before exit 1.

## Files to Read First

- `src/commands/archive_tasks.rs` — the command wrapper (this module)
- `src/archive.rs` — `archive_tasks` + `ArchiveResult { tasks_path, archived_count }`, where the real parsing/rewriting lives
- `src/config.rs` — `load_config` / `specs_dir` resolution

## Current Status

Text, JSON/`--json`, Markdown/GitHub, adversarial paths, literal Unix backslashes, and fail-closed exit behavior are covered; the delegate independently proves the transactional filesystem boundaries.

## Notes

- `ArchiveResult.tasks_path` is a repo-relative `PathBuf`; structured renderers normalize separators only on Windows while text preserves terminal-native output.
- The delegate moves completed items into `## Archive`; the command wrapper only reports that result.
- Markdown table paths entity-encode table pipes when needed, otherwise use a delimiter longer than every contained backtick run, and visibly encode line/control/bidirectional controls.
