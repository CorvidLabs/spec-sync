---
module: archive
version: 4
status: stable
files:
  - src/archive.rs
db_tables: []
tracks: [94]
depends_on:
  - specs/validator/validator.spec.md
---

# Archive

## Purpose

Moves completed markdown task items (`- [x]`) from active sections of companion `tasks.md` files into an `## Archive` section at the bottom. Keeps task history accessible without cluttering the active task list.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `archive_tasks` | `root: &Path, specs_dir: &Path, dry_run: bool` | `ArchiveReport` | Plan every companion update, stage replacements, and atomically publish a complete archival report |
| `count_completed_tasks` | `specs_dir: &Path` | `usize` | Count all completed tasks across all tasks.md files |

### Exported Structs

| Type | Description |
|------|-------------|
| `ArchiveResult` | One planned, successful, or rolled-back file operation with a path-safe `PathBuf` and archived-task count |
| `ArchiveFailure` | Structured companion path, filesystem operation, and error detail for an unsuccessful operation |
| `ArchiveOperation` | Stable operation identity: inspect, read, preflight, stage, publish, or rollback |
| `ArchiveReport` | Invocation-wide dry-run flag plus planned, succeeded, rolled-back, and failed operation collections |

## Invariants

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

## Behavioral Examples

### Scenario: Archive completed tasks

- **Given** a tasks.md file with 3 completed and 2 pending items
- **When** `archive_tasks(root, specs_dir, false)` is called
- **Then** the 3 completed items move to `## Archive`, 2 pending items remain in place

### Scenario: Dry run

- **Given** tasks.md files with completed items
- **When** `archive_tasks(root, specs_dir, true)` is called
- **Then** returns the planned `ArchiveResult` entries in a complete `ArchiveReport` but does not stage or modify files

### Scenario: No completed tasks

- **Given** all tasks.md files have only pending items
- **When** `archive_tasks` is called
- **Then** returns a complete report with empty operation collections

### Scenario: One candidate cannot be read

- **Given** one valid candidate and one unreadable or non-UTF-8 tasks.md file
- **When** `archive_tasks` applies the invocation
- **Then** the read failure is structured in the report and no destination is modified

## Error Cases

| Condition | Behavior |
|-----------|----------|
| tasks.md inspection/read failure | Records the path, operation, and error; returns an incomplete report without staging or publishing any destination |
| replacement staging failure | Removes staged temporary files and returns an incomplete report without publishing any destination |
| atomic publication failure | Records the failure, rolls back prior publishes when safe, and reports any destination that could not be restored as partial |
| symbolic-link or non-file tasks path | Rejects it during preflight without replacing the entry |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| validator | `find_spec_files` to locate spec files and their companions |

### Consumed By

| Module | What is used |
|--------|-------------|
| main | `archive_tasks` via `cmd_archive_tasks` subcommand |

## Change Log

| Date | Change |
|------|--------|
| 2026-07-26 | Replace silent partial writes with typed plan/stage/publish reports, atomic same-directory replacement, permission preservation, and rollback |
| 2026-04-10 | Populated requirements.md with user stories, acceptance criteria, constraints, and out-of-scope items |
| 2026-04-06 | Initial spec for v3.3.0 |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
