---
module: cmd_scaffold
version: 1
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

1. Both scan source dirs for module name matches
2. `cmd_scaffold` supports custom templates and auto-appends to registry
3. Neither overwrites existing specs
4. Companion files always generated

## Behavioral Examples

### Scenario: Scaffold with auto-detection

- **Given** `src/auth.rs` exists
- **When** `cmd_add_spec(root, "auth")` runs
- **Then** creates spec with detected sources and companions

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Spec exists | Early return |
| Dir creation fails | Exits 1 |
| Custom template dir missing | Falls back to built-in |

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
| 2026-04-09 | Initial spec |
