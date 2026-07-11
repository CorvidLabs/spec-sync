## ADDED

### REQUIREMENT REQ-cmd-archive-tasks-001

The archive-tasks command SHALL delegate task archival safely and SHALL distinguish dry-run, no-op, per-file, and summary output.

Acceptance Criteria
- `cmd_archive_tasks(root, dry_run)` loads config, resolves `config.specs_dir` under `root`, and delegates archiving to `archive::archive_tasks(root, &specs_dir, dry_run)`
- When `dry_run` is true, an informational banner ("Dry run — no files will be modified") prints and no files are written; per-file lines read "would archive"
- When `dry_run` is false, completed tasks are moved and per-file lines read "archived"
- When the delegate returns no results, prints "No completed tasks to archive." and returns without a summary
- Each affected file prints its relative `tasks_path` and `archived_count`; a trailing summary reports the summed task count across the affected file count
