## ADDED

### REQUIREMENT REQ-archive-001

The archive module SHALL move only completed task items into preserved archive sections while keeping preview and error handling deterministic.

Acceptance Criteria
- Only task items matching `- [x]` or `- [X]` (case-insensitive) are eligible for archiving
- An `## Archive` section is automatically created at the bottom of tasks.md if it does not exist
- Existing archive content is preserved and new completed tasks are appended to it
- `dry_run: true` returns `ArchiveResult` entries for all files that would be modified without writing any changes to disk
- Files with no completed tasks are excluded from the results vector
- The `count_completed_tasks` function returns the total count of `- [x]` items across all tasks.md files in the specs directory
- `ArchiveResult` entries include the relative path to the tasks.md file and the count of tasks archived
- File permission errors (unreadable/unwritable) print a red error message and continue processing remaining files
- Task items use the exact markdown format `- [x] ` (with space after bracket) to be recognized as completed
