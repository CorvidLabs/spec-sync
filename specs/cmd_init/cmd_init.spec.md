---
module: cmd_init
version: 1
status: stable
files:
  - src/commands/init.rs
db_tables: []
tracks: []
depends_on:
  - specs/config/config.spec.md
---

# Cmd Init

## Purpose

Implements the `specsync init` command. Creates a `specsync.json` configuration file with auto-detected source directories.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_init` | `root: &Path` | `()` | Create specsync.json with auto-detected source dirs |
| `ensure_hashes_gitignored` | `root: &Path` | `Result<bool, std::io::Error>` | Add hashes.json to .specsync/.gitignore and .specsync/hashes.json to root .gitignore; returns Ok(true) if changes were made |

## Invariants

1. Auto-detects source directories via `config::detect_source_dirs()`
2. Will not overwrite existing `specsync.json`
3. Writes default config with detected dirs and standard required sections

## Behavioral Examples

### Scenario: First init

- **Given** no `specsync.json` exists
- **When** `cmd_init(root)` runs
- **Then** creates config with detected source dirs

### Scenario: Config exists

- **Given** `specsync.json` already exists
- **When** `cmd_init(root)` runs
- **Then** prints message and returns without changes

## Error Cases

| Condition | Behavior |
|-----------|----------|
| File write fails | Exits 1 |
| No source dirs detected | Creates config with empty `sourceDirs` |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| config | `detect_source_dirs` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync init` |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-09 | Initial spec |
