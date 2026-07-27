---
spec: cmd_archive_tasks.spec.md
---

## User Stories

- As a developer, I want completed (`- [x]`) tasks moved out of each companion `tasks.md` into a `## Archive` section so active task lists stay focused on outstanding work
- As a maintainer, I want a `--dry-run` preview so I can see which files and how many tasks would be archived before committing to a write
- As a script author, I want a grammatically correct summary line ("Archived 1 task across 1 file", "Archived N tasks across M files", or "No completed tasks to archive.") so the result is easy to read or assert on
- As an automation author, I want equivalent `--format json` and `--json` output so archive results are safely parseable
- As a reviewer, I want `--format markdown` output with a result table suitable for a PR comment
- As a CI operator, I want incomplete operations to render their structured failures and exit 1 so partial or inconclusive archival cannot pass
- As a user with unusual legal filenames, I want every output format to preserve path identity without Markdown row/code-span injection

## Acceptance Criteria

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
- Markdown/GitHub paths cannot inject rows or break code spans through pipes, backtick runs, line/control characters, or bidirectional controls
- Text output uses task/tasks and file/files according to the actual counts
- Text paths and errors visibly encode line/control and bidirectional-control characters rather than emitting terminal-control content

## Constraints

- Pure orchestration wrapper: all task-parsing/rewriting logic lives in `archive::archive_tasks`; this module only loads config and formats output
- Must not panic or falsely succeed on missing/unreadable `tasks.md` files — the underlying report retains structured failures
- Text output uses `colored` for status glyphs (`ℹ`, `✓`); structured output contains no ANSI formatting
- The command must flush the rendered failure report before exiting 1

## Out of Scope

- The actual task-parsing/rewriting logic (owned by the `archive` module)
- Restoring archived tasks back into the active list
- Interactive prompts or GUI

### REQ-cmd-archive-tasks-001

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
- Markdown/GitHub paths cannot inject rows or break code spans through pipes, backtick runs, line/control characters, or bidirectional controls
- Text output uses task/tasks and file/files according to the actual counts
- Text paths and errors visibly encode line/control and bidirectional-control characters rather than emitting terminal-control content
