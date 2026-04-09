---
module: cmd_init_registry
version: 1
status: stable
files:
  - src/commands/init_registry.rs
db_tables: []
tracks: []
depends_on:
  - specs/config/config.spec.md
  - specs/registry/registry.spec.md
---

# Cmd Init Registry

## Purpose

Implements the `specsync init-registry` command. Creates a `specsync-registry.toml` for cross-project spec references with auto-detected entries.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_init_registry` | `root: &Path, name: Option<String>` | `()` | Generate registry TOML with spec entries |

## Invariants

1. Delegates to `registry::generate_registry()`
2. Will not overwrite existing `specsync-registry.toml`
3. `--name` overrides auto-detected project name

## Behavioral Examples

### Scenario: Generate registry

- **Given** 25 specs, no existing registry
- **When** `cmd_init_registry(root, None)` runs
- **Then** creates TOML with 25 entries

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Registry exists | Early return |
| Write fails | Exits 1 |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| config | `load_config` |
| registry | `generate_registry` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync init-registry` |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-09 | Initial spec |
