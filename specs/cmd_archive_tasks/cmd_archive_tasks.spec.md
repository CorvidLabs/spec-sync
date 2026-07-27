---
module: cmd_archive_tasks
version: 6
status: stable
files:
  - src/commands/archive_tasks.rs
db_tables: []
tracks: []
depends_on:
  - specs/archive/archive.spec.md
  - specs/config/config.spec.md
  - specs/types/types.spec.md
---

# Cmd Archive Tasks

## Purpose

Implements the `specsync archive-tasks` command. Moves completed tasks (checked items) from companion tasks.md files into an archive section, keeping active task lists clean.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_archive_tasks` | `root: &Path, dry_run: bool, format: OutputFormat` | `()` | Archive completed tasks and render text, JSON, or Markdown results |

## Invariants

1. Delegates entirely to `archive::archive_tasks()` for the actual archiving logic
2. Dry-run mode prints affected files but makes no writes
3. Gracefully handles empty results (no completed tasks to archive)

## Behavioral Examples

### Scenario: Tasks archived successfully

- **Given** tasks.md has 3 checked items (`- [x]`)
- **When** `cmd_archive_tasks(root, false, OutputFormat::Text)` is called
- **Then** checked items move to the `## Archive` section and count is printed

### Scenario: Dry run

- **Given** tasks.md has completed items
- **When** `cmd_archive_tasks(root, true, OutputFormat::Text)` is called
- **Then** prints what would be archived without modifying files

### Scenario: Machine-readable dry run

- **Given** tasks.md has completed items
- **When** `cmd_archive_tasks` runs with JSON format and `dry_run = true`
- **Then** stdout is one valid JSON document with `would_change: true`, `applied: false`, and no file is modified

### Scenario: Apply cannot read one candidate

- **Given** one valid archive candidate and one malformed tasks.md file
- **When** `cmd_archive_tasks` runs in apply mode
- **Then** stdout reports the plan and structured read failure, all destinations remain unchanged, and the process exits 1

## Error Cases

| Condition | Behavior |
|-----------|----------|
| No tasks.md files found | Prints "nothing to archive" |
| No completed tasks | Prints "nothing to archive" |
| Incomplete plan, stage, publish, or rollback | Renders the selected format with `complete: false`/failure details and exits 1 |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| archive | `archive_tasks` |
| config | `load_config` |
| types | `OutputFormat` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync archive-tasks` |

## Change Log

| Date | Change |
|------|--------|
| 2026-07-26 | Preserve every legal Unix backslash parity, including backslash-before-pipe paths, through one safe Markdown/GitHub code cell |
| 2026-07-26 | Add truthful typed failure output, exit-1 incomplete results, safe path rendering, and singular/plural text |
| 2026-07-26 | Normalize structured result paths to portable `/` separators on Windows |
| 2026-07-26 | Fix #417 structured output: parse-clean JSON, Markdown/GitHub tables, shorthand equivalence, and explicit dry-run truth |
| 2026-04-09 | Initial spec |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
