## MODIFIED

### REQUIREMENT REQ-archive-001

The archive module SHALL move only completed task items into preserved archive sections through a deterministic, typed, transactionally safe invocation.

Acceptance Criteria
- Only task items matching `- [x]` or `- [X]` (case-insensitive) are eligible for archiving
- An `## Archive` section is automatically created at the bottom of tasks.md if it does not exist
- Existing archive content is preserved and new completed tasks are appended to it
- `dry_run: true` returns an `ArchiveReport` whose planned operations exactly match an apply plan without staging or writing any destination
- Files with no completed tasks are excluded from the results vector
- The `count_completed_tasks` function returns the total count of `- [x]` items across all tasks.md files in the specs directory
- `ArchiveResult` entries retain a relative `PathBuf` and the count of tasks archived
- `ArchiveReport` separates planned, succeeded, rolled-back, and failed operations
- Every `ArchiveFailure` includes the path, operation identity, and error detail
- Any planning or staging failure prevents all destination-file writes
- Replacements are staged in each destination directory, preserve original permissions, and publish atomically
- A late publish failure rolls back prior publishes when safe and truthfully retains any unrolled-back operation in `succeeded`
- Task items use the exact markdown format `- [x] ` (with space after bracket) to be recognized as completed

### SPEC SECTION Invariants

1. Only items matching `- [x]` or `- [X]` (case-insensitive) are archived
2. If no `## Archive` section exists, one is created at the bottom of the file
3. Existing archive content is preserved — new items are appended
4. `dry_run: true` returns the exact plan without staging or modifying files
5. Files with no completed tasks are skipped (not included in results)
6. Uses `find_spec_files` from validator to discover specs and their companion files
7. Every candidate is read and preflighted before staging; any planning failure prevents all destination writes
8. Every replacement is staged in its destination directory before the first destination is published
9. Staged files preserve original permissions and are atomically renamed over their destination
10. A late publish failure is reported; prior successful publishes are rolled back when safe, and any remaining changed files are exposed as partial success
