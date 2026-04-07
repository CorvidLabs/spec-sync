---
module: archive
version: 1
status: stable
files:
  - src/archive.rs
db_tables: []
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
| `archive_tasks` | `root: &Path, specs_dir: &Path, dry_run: bool` | `Vec<ArchiveResult>` | Scan all spec companion tasks.md files, move completed tasks to archive section |
| `count_completed_tasks` | `specs_dir: &Path` | `usize` | Count all completed tasks across all tasks.md files |

### Exported Structs

| Type | Description |
|------|-------------|
| `ArchiveResult` | Result of archiving tasks in a single file — contains `tasks_path: String` and `archived_count: usize` |

## Invariants

1. Only items matching `- [x]` or `- [X]` (case-insensitive) are archived
2. If no `## Archive` section exists, one is created at the bottom of the file
3. Existing archive content is preserved — new items are appended
4. `dry_run: true` returns results without modifying files
5. Files with no completed tasks are skipped (not included in results)
6. Uses `find_spec_files` from validator to discover specs and their companion files

## Behavioral Examples

### Scenario: Archive completed tasks

- **Given** a tasks.md file with 3 completed and 2 pending items
- **When** `archive_tasks(root, specs_dir, false)` is called
- **Then** the 3 completed items move to `## Archive`, 2 pending items remain in place

### Scenario: Dry run

- **Given** tasks.md files with completed items
- **When** `archive_tasks(root, specs_dir, true)` is called
- **Then** returns `ArchiveResult` entries but does not modify any files

### Scenario: No completed tasks

- **Given** all tasks.md files have only pending items
- **When** `archive_tasks` is called
- **Then** returns an empty vec

## Error Cases

| Condition | Behavior |
|-----------|----------|
| tasks.md file unreadable | Prints error in red, continues processing other files |
| tasks.md file unwritable | Prints error in red, continues processing other files |

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
| 2026-04-06 | Initial spec for v3.3.0 |
