---
module: cmd_archive_tasks
version: 1
status: stable
files:
  - src/commands/archive_tasks.rs
db_tables: []
tracks: []
depends_on:
  - specs/archive/archive.spec.md
  - specs/config/config.spec.md
---

# Cmd Archive Tasks

## Purpose

Implements the `specsync archive-tasks` command. Moves completed tasks (checked items) from companion tasks.md files into an archive section, keeping active task lists clean.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_archive_tasks` | `root: &Path, dry_run: bool` | `()` | Archive completed tasks from all spec companion tasks.md files |

## Invariants

1. Delegates entirely to `archive::archive_tasks()` for the actual archiving logic
2. Dry-run mode prints affected files but makes no writes
3. Gracefully handles empty results (no completed tasks to archive)

## Behavioral Examples

### Scenario: Tasks archived successfully

- **Given** tasks.md has 3 checked items (`- [x]`)
- **When** `cmd_archive_tasks(root, false)` is called
- **Then** checked items move to `## Done` section and count is printed

### Scenario: Dry run

- **Given** tasks.md has completed items
- **When** `cmd_archive_tasks(root, true)` is called
- **Then** prints what would be archived without modifying files

## Error Cases

| Condition | Behavior |
|-----------|----------|
| No tasks.md files found | Prints "nothing to archive" |
| No completed tasks | Prints "nothing to archive" |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| archive | `archive_tasks` |
| config | `load_config` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync archive-tasks` |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-09 | Initial spec |
