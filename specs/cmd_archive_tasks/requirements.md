---
spec: cmd_archive_tasks.spec.md
---

## User Stories

- As a developer, I want completed (`- [x]`) tasks moved out of each companion `tasks.md` into a `## Done` section so active task lists stay focused on outstanding work
- As a maintainer, I want a `--dry-run` preview so I can see which files and how many tasks would be archived before committing to a write
- As a script author, I want a clear summary line ("Archived N task(s) across M file(s)", or "No completed tasks to archive.") so the result is easy to read or assert on

## Acceptance Criteria

- `cmd_archive_tasks(root, dry_run)` loads config, resolves `config.specs_dir` under `root`, and delegates archiving to `archive::archive_tasks(root, &specs_dir, dry_run)`
- When `dry_run` is true, an informational banner ("Dry run — no files will be modified") prints and no files are written; per-file lines read "would archive"
- When `dry_run` is false, completed tasks are moved and per-file lines read "archived"
- When the delegate returns no results, prints "No completed tasks to archive." and returns without a summary
- Each affected file prints its relative `tasks_path` and `archived_count`; a trailing summary reports the summed task count across the affected file count

## Constraints

- Pure orchestration wrapper: all task-parsing/rewriting logic lives in `archive::archive_tasks`; this module only loads config and formats output
- Must not panic on missing/unreadable `tasks.md` files — the underlying module skips them gracefully
- Output uses `colored` for status glyphs (`ℹ`, `✓`)

## Out of Scope

- The actual task-parsing/rewriting logic (owned by the `archive` module)
- Restoring archived tasks back into the active list
- Interactive prompts or GUI
