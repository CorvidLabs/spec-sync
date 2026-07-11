---
module: cmd_init_registry
version: 2
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

Implements the `specsync init-registry` command. Creates a registry file for cross-project spec references with auto-detected entries. The registry is written to the v4 location (`.specsync/registry.toml`); the legacy root-level `specsync-registry.toml` is only used for un-migrated 3.x layouts.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_init_registry` | `root: &Path, name: Option<String>` | `()` | Generate registry TOML with spec entries |

## Invariants

1. Delegates to `registry::generate_registry()` and resolves the target path via `registry::local_registry_path()`
2. Will not overwrite an existing registry file (v4 or legacy location)
3. `--name` overrides auto-detected project name
4. Never recreates the legacy root-level registry in a v4 project — doing so would re-trigger the legacy-layout migration nag

## Behavioral Examples

### Scenario: Generate registry

- **Given** 25 specs, no existing registry
- **When** `cmd_init_registry(root, None)` runs
- **Then** creates TOML with 25 entries at `.specsync/registry.toml` (or `specsync-registry.toml` on a legacy 3.x layout)

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
| registry | `generate_registry`, `local_registry_path` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync init-registry` |

## Change Log

| Date | Change |
|------|--------|
| 2026-04-09 | Initial spec |
| 2026-06-11 | v2: Write to v4 `.specsync/registry.toml` (legacy root path only for un-migrated 3.x projects) |
