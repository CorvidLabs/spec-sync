---
spec: archive.spec.md
---

## User Stories

- As a developer, I want completed tasks to be automatically archived so that my active task list stays uncluttered and focused on pending work
- As a team lead, I want to preview which tasks would be archived before committing changes so that I can verify important completed tasks are properly documented
- As a spec maintainer, I want archived tasks preserved in a dedicated section so that I retain historical context and decision records
- As a CI operator, I want the archive operation to continue processing all files even if some fail so that a single corrupted tasks.md doesn't block the entire operation
- As a developer, I want to count completed tasks across all specs so that I can track team velocity and spec completion progress

## Acceptance Criteria

- Only task items matching `- [x]` or `- [X]` (case-insensitive) are eligible for archiving
- An `## Archive` section is automatically created at the bottom of tasks.md if it does not exist
- Existing archive content is preserved and new completed tasks are appended to it
- `dry_run: true` returns `ArchiveResult` entries for all files that would be modified without writing any changes to disk
- Files with no completed tasks are excluded from the results vector
- The `count_completed_tasks` function returns the total count of `- [x]` items across all tasks.md files in the specs directory
- `ArchiveResult` entries include the relative path to the tasks.md file and the count of tasks archived
- File permission errors (unreadable/unwritable) print a red error message and continue processing remaining files
- Task items use the exact markdown format `- [x] ` (with space after bracket) to be recognized as completed

## Constraints

- Must use `find_spec_files` from the validator module to discover spec directories and their companion tasks.md files
- Archive section header must be exactly `## Archive` (case-sensitive) to match expected markdown structure
- Task items must remain in their original order within the archive section as they are appended
- Must not modify files when `dry_run` is enabled — results should reflect what would happen without side effects
- Must gracefully handle missing or malformed tasks.md files without panicking
- All file paths in `ArchiveResult` must be relative to the specs directory for portability

## Out of Scope

- Re-archiving tasks that are already in the archive section
- Restoring archived tasks back to active sections
- Automatic archiving on a schedule or via git hooks
- Configuring custom archive section headers or formats
- Partial task archiving (e.g., archiving by date range or category)
- Backup creation before archiving operations