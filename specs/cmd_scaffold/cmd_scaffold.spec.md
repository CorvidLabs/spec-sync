---
module: cmd_scaffold
version: 5
status: stable
files:
  - src/commands/scaffold.rs
db_tables: []
tracks: []
depends_on:
  - specs/config/config.spec.md
  - specs/exports/exports.spec.md
  - specs/generator/generator.spec.md
  - specs/registry/registry.spec.md
---

# Cmd Scaffold

## Purpose

Implements `specsync add-spec` and `specsync scaffold` commands. Creates new spec files from templates with auto-detected source files and companion files. `scaffold` adds custom dir/template support and auto-registration.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_add_spec` | `root: &Path, module_name: &str` | `()` | Create full spec from built-in template with companions |
| `cmd_scaffold` | `root: &Path, module_name: &str, dir: Option<PathBuf>, template: Option<PathBuf>` | `()` | Scaffold with optional custom dir/template and auto-registration |

## Invariants

1. Both scan source dirs for module name matches; `cmd_scaffold` falls back to the project's single non-test source file (e.g. only `src/lib.rs`) when no name match exists
2. `cmd_scaffold` supports custom templates and auto-appends to registry
3. Neither overwrites existing specs
4. Companion files (tasks.md, context.md, requirements.md, testing.md) are always generated with guided starter content; design.md is generated only when `companions.design` is enabled in config
5. Both validate `module_name` as a single path segment before any filesystem write — a name containing a path separator (`/`, `\`), `.`/`..`, or an absolute path is refused with exit 1, so scaffolding can never create files outside the project (no path traversal)

## Behavioral Examples

### Scenario: Scaffold with auto-detection

- **Given** `src/auth.rs` exists
- **When** `cmd_add_spec(root, "auth")` runs
- **Then** creates spec with detected sources and companions (including design.md if `companions.design` is enabled)

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Spec exists | Early return |
| Dir creation fails | Exits 1 |
| Custom template dir missing | Falls back to built-in |
| Module name with path separator / `..` / absolute path | Refused before any write; prints `invalid module name …` and exits 1 |
| Empty module name | Refused; exits 1 |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| config | `load_config` |
| exports | `get_exported_symbols` |
| generator | `generate_companion_files` |
| registry | `append_to_registry` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync add-spec` and `specsync scaffold` |

## Change Log

| Date | Change |
|------|--------|
| 2026-06-11 | `cmd_scaffold` falls back to the project's single source file when no module name match exists |
| 2026-06-07 | Document guided starter content in generated companions |
| 2026-04-09 | Initial spec |
| 2026-04-13 | Document companions.design flag for conditional design.md generation |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-14 | CHG-0038-harden-commonjs-export-extraction-and-exclude-module-javascript-tests-from-gener: Harden CommonJS export extraction and exclude module JavaScript tests from generated specs |
