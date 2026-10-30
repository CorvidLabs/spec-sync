---
module: cmd_new
version: 11
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

1. Uses shared confined source selection for exact regular files and directory name matches; explicit module mappings take precedence, including empty mappings. When no explicit mapping or name match exists, exactly one distinct eligible source file is used, with overlapping selections deduplicated.
2. Extracts exports to pre-populate Public API tables via `generator::generate_spec`
3. `--full` generates companion files (tasks.md, context.md, requirements.md, testing.md) via `generator::generate_companion_files_for_spec()`; design.md is included only when `companions.design` is enabled in config
4. Emits the same seven-section skeleton as `add-spec` / `generate`; Change Log dates come from the shared generator, not a private `chrono_lite_today()` helper
5. Will not overwrite existing spec
6. Module names are refused under the same rules as `scaffold` (`validate_scaffold_module_name` plus case-collision)

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
| Invalid module name (path separator, `.`/`..`, absolute/drive-relative, control chars, reserved names, leading dashes, spaces) | Refused via `validate_scaffold_module_name` before any write; prints `invalid module name …` and exits 1 (no path traversal) |
| Case-fold collision with an existing spec directory | Refused via `check_case_collision` before any write |

## Dependencies

#### Consumes

| Module | What is used |
|--------|-------------|
| config | `load_config` |
| generator | `generate_spec`, `collect_exports_for_files`, `generate_companion_files_for_spec`, `find_new_module_sources` |
| commands | `validate_scaffold_module_name`, `check_case_collision` |

#### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync new` |

## Change Log

| Date | Change |
|------|--------|
| 2026-10-14 | Approved exact-source follow-up: confined exact-file selection, shared filters, overlap deduplication and explicit-mapping precedence; no parent-scan widening |
| 2026-10-10 | Add `cmd_new.3md` beside the spec. The first plane is context and later planes are lessons. `context.md` stays. |
| 2026-06-11 | Fall back to the project's single source file when no name match exists (README quickstart flow); warn instead of silently writing an empty `files:` list |
| 2026-06-07 | Replace unfinished-marker generated rows with review prompts |
| 2026-04-09 | Initial spec |
| 2026-04-13 | Document testing.md and conditional design.md in companion generation |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-14 | CHG-0038-harden-commonjs-export-extraction-and-exclude-module-javascript-tests-from-gener: Harden CommonJS export extraction and exclude module JavaScript tests from generated specs |
| 2026-08-15 | CHG-0128-every-command-that-derives-a-module-s-api-must-honour-the-configured-export-leve: Every command that derives a module's API must honour the configured export level and parse mode, so check, score, new, generate, scaffold and diff cannot disagree about what the API is |
| 2026-09-09 | close-the-specsync-6-0-0-p1-release-defects-found-in-overnight-proving: Close the SpecSync 6.0.0 P1 release defects found in overnight proving |
| 2026-09-09 | stop-check-fix-from-overwriting-fenced-public-api-examples-and-restore-git-discovery-for-a-project-inside-a-repository: Stop check --fix from overwriting fenced Public API examples and restore git discovery for a project inside a repository subdirectory |
| 2026-10-15 | complete-exact-cargo-source-handling-and-scoped-batch-recognition-for-specsync-6-1-1: Complete exact Cargo source handling and scoped batch recognition for SpecSync 6.1.1 |
