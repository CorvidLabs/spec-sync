---
module: cmd_new
version: 1
status: stable
files:
  - src/commands/new.rs
db_tables: []
tracks: []
depends_on:
  - specs/config/config.spec.md
  - specs/exports/exports.spec.md
  - specs/generator/generator.spec.md
---

# Cmd New

## Purpose

Implements the `specsync new` command. Quick-creates a minimal spec with auto-detected source files and pre-populated exports. `--full` also generates companion files.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_new` | `root: &Path, module_name: &str, full: bool` | `()` | Create a new spec with auto-detected sources and pre-populated Public API |

## Invariants

1. Auto-detects source files by scanning source dirs for module name matches
2. Extracts exports to pre-populate Public API tables
3. `--full` generates companion files via `generator::generate_companion_files()`
4. Includes custom `chrono_lite_today()` for dates without chrono dependency
5. Will not overwrite existing spec

## Behavioral Examples

### Scenario: Quick spec

- **Given** `src/auth.rs` exists
- **When** `cmd_new(root, "auth", false)` runs
- **Then** creates `specs/auth/auth.spec.md` with detected source and exports

### Scenario: Full with companions

- **Given** `--full` flag
- **When** `cmd_new` runs
- **Then** creates spec.md, tasks.md, context.md, requirements.md

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Spec already exists | Exits 1 |
| No source files found | Creates spec with empty `files:` |
| Dir creation fails | Exits 1 |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| config | `load_config` |
| exports | `get_exported_symbols`, `has_extension` |
| generator | `generate_companion_files` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync new` |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-09 | Initial spec |
