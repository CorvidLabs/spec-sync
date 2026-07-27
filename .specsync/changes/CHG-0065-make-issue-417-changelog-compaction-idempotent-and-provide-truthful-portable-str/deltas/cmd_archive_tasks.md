## MODIFIED

### REQUIREMENT REQ-cmd-archive-tasks-001

The archive-tasks command SHALL delegate task archival safely and SHALL render truthful dry-run, success, partial, rollback, and failure outcomes.

Acceptance Criteria
- `cmd_archive_tasks(root, dry_run, format)` loads config, resolves `config.specs_dir` under `root`, and delegates archiving to `archive::archive_tasks(root, &specs_dir, dry_run)`
- When `dry_run` is true, an informational banner ("Dry run — no files will be modified") prints and no files are written; per-file lines read "would archive"
- When `dry_run` is false, completed tasks are moved and per-file lines read "archived"
- When the delegate returns no results, prints "No completed tasks to archive." and returns without a summary
- Each affected file prints its relative `tasks_path` and `archived_count`; a trailing summary reports the summed task count across the affected file count
- JSON mode emits one ANSI-free document with command, dry-run, `would_change`, `applied`, aggregate counts, and per-file results
- In dry-run JSON, `would_change` reflects selected changes while `applied` remains false
- JSON separates planned, succeeded, rolled-back, and failed operations and exposes truthful `complete` and `partial` booleans
- Incomplete apply results render before exit 1 and never report `applied: true`
- `--json` is byte-equivalent to `--format json`
- Markdown and GitHub modes emit equivalent headings, optional dry-run notices, result/failure tables, and truthful singular/plural summaries
- Structured Windows paths use `/` separators; legal Unix backslashes remain literal
- Markdown/GitHub paths cannot inject rows or break their single code element through pipes, backtick runs, line/control characters, or bidirectional controls
- Text output uses task/tasks and file/files according to the actual counts
- Text paths and errors visibly encode line/control and bidirectional-control characters rather than emitting terminal-control content

### SPEC SECTION Invariants

1. Delegates entirely to `archive::archive_tasks()` for planning and transactional archival
2. Dry-run mode prints affected files but makes no writes
3. Gracefully handles empty results (no completed tasks to archive)
4. JSON is one parseable, ANSI-free document; `--json` and `--format json` are equivalent.
5. Markdown and GitHub formats render a heading, dry-run notice, result table, and truthful summary.
6. Structured dry-run output distinguishes `would_change: true` from `applied: false`.
7. Structured paths retain `PathBuf` identity until rendering: Windows separators become `/`, while literal Unix backslashes remain literal
8. JSON exposes `complete`, `partial`, and explicit planned/succeeded/rolled-back/failed operation arrays
9. Any incomplete report is fully rendered before the command exits 1; `applied` is never true for an incomplete invocation
10. Markdown/GitHub paths use one code element: dynamic-backtick spans normally and entity-safe HTML code for literal-pipe paths, plus visible escapes for control and bidirectional-control characters; every legal Unix backslash parity remains unchanged
11. Text output uses correct task/tasks and file/files labels
12. Text paths and filesystem errors visibly escape control and bidirectional-control characters
