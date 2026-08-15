---
module: cmd_new
version: 6
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

Implements the `specsync new` command. Quick-creates a minimal spec with auto-detected source files and pre-populated exports that include generated review prompts. `--full` also generates companion files.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_new` | `root: &Path, module_name: &str, full: bool` | `()` | Create a new spec with auto-detected sources and pre-populated Public API |

## Invariants

1. Auto-detects source files by scanning source dirs for module name matches; when nothing matches and the project has exactly one non-test source file (e.g. only `src/lib.rs`), that file is used as the module's source
2. Extracts exports to pre-populate Public API tables
3. `--full` generates companion files (tasks.md, context.md, requirements.md, testing.md) via `generator::generate_companion_files_for_spec()`; design.md is included only when `companions.design` is enabled in config
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
- **Then** creates spec.md, tasks.md, context.md, requirements.md, testing.md (and design.md if `companions.design` is enabled)

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Spec already exists | Exits 1 |
| No source files found | Creates spec with empty `files:` and prints a ⚠ explaining that the `files:` list must be filled in before `check` passes |
| Dir creation fails | Exits 1 |
| Invalid module name (path separator, `.`/`..`, absolute/drive-relative, control chars) | Refused via `validate_module_name` before any write; prints `invalid module name …` and exits 1 (no path traversal) |

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
| 2026-06-11 | Fall back to the project's single source file when no name match exists (README quickstart flow); warn instead of silently writing an empty `files:` list |
| 2026-06-07 | Replace unfinished-marker generated rows with review prompts |
| 2026-04-09 | Initial spec |
| 2026-04-13 | Document testing.md and conditional design.md in companion generation |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-14 | CHG-0038-harden-commonjs-export-extraction-and-exclude-module-javascript-tests-from-gener: Harden CommonJS export extraction and exclude module JavaScript tests from generated specs |
| 2026-08-15 | CHG-0128-every-command-that-derives-a-module-s-api-must-honour-the-configured-export-leve: Every command that derives a module's API must honour the configured export level and parse mode, so check, score, new, generate, scaffold and diff cannot disagree about what the API is |
