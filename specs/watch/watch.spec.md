---
module: watch
version: 1
status: stable
files:
  - src/watch.rs
db_tables: []
tracks: [114]
depends_on:
  - specs/config/config.spec.md
---

# Watch

## Purpose

File watcher that re-runs `specsync check` on file changes. Watches spec and source directories with 500ms debounce, providing live validation feedback during development.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `run_watch` | `root, strict, require_coverage` | `()` | Run check in watch mode — initial run, then re-run on file changes |

## Invariants

1. Debounce interval is 500ms — prevents rapid-fire re-runs during batch saves
2. Additional 300ms minimum between runs as extra debounce
3. Only reacts to Create, Modify, and Remove events — ignores access and metadata changes
4. Queued events are drained after each check run to prevent cascading re-runs
5. Watch exits if no directories exist to watch (both specs_dir and all source_dirs missing)
6. Check is run as a child process (fork of current executable) to isolate exit calls
7. Screen is cleared before each re-run for clean output
8. Changed file path is displayed in the separator header

## Behavioral Examples

### Scenario: Initial run

- **Given** a project with specs and source directories
- **When** `run_watch` is called
- **Then** runs `specsync check` immediately, then watches for changes

### Scenario: File modification triggers re-check

- **Given** watch mode is running
- **When** a `.spec.md` file is modified
- **Then** re-runs check after 500ms debounce, showing the changed file path

### Scenario: Rapid saves

- **Given** watch mode is running
- **When** multiple files are saved within 500ms
- **Then** only one check run is triggered (debounced)

## Error Cases

| Condition | Behavior |
|-----------|----------|
| No directories to watch | Prints error, exits with code 1 |
| Watcher creation fails | Panics with "Failed to create file watcher" |
| Individual dir watch fails | Prints warning, continues watching other dirs |
| Check command fails | Prints "Some checks failed", continues watching |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| config | `load_config` |
| notify | File system event watching with debounce |

### Consumed By

| Module | What is used |
|--------|-------------|
| main | `run_watch` (via `watch` subcommand) |

## Change Log

| Date | Change |
|------|--------|
| 2026-03-25 | Initial spec |
