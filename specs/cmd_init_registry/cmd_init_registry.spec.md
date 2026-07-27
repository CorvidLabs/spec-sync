---
module: cmd_init_registry
version: 4
status: stable
files:
  - src/commands/init_registry.rs
db_tables: []
implements: [440]
tracks: []
depends_on:
  - specs/config/config.spec.md
  - specs/registry/registry.spec.md
---

# Cmd Init Registry

## Purpose

Implements the `specsync init-registry` command. Creates a safely serialized registry for cross-project spec references, validates selected config before writing, preserves existing registries, and emits truthful format-aware outcomes.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_init_registry` | `root: &Path, name: Option<String>, format: OutputFormat` | `()` | Validate inputs/config, generate registry TOML once, and render a format-aware outcome |

## Invariants

1. Delegates to `registry::generate_registry()` and resolves the target path via `registry::local_registry_path()`
2. Will not overwrite an existing registry file (v4 or legacy location)
3. `--name` overrides auto-detected project name
4. Never recreates the legacy root-level registry in a v4 project — doing so would re-trigger the legacy-layout migration nag
5. Empty/whitespace-only names fail before any write; arbitrary non-empty names are TOML-serialized without interpolation.
6. Existing valid registries are byte-identical no-op successes with `created = false` and `unchanged = true`; existing invalid/inert files fail visibly.
7. JSON, Markdown/GitHub, table, and CSV output carry the same created/unchanged/error semantics.

## Behavioral Examples

### Scenario: Generate registry

- **Given** 25 specs, no existing registry
- **When** `cmd_init_registry(root, None, Text)` runs
- **Then** creates TOML with 25 entries at `.specsync/registry.toml` (or `specsync-registry.toml` on a legacy 3.x layout)

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Registry exists | Early return |
| Registry exists but is malformed or inert | Exits 1 without overwriting |
| Empty/whitespace-only `--name` | Exits 1 before creating the registry |
| Selected config is malformed or has wrong known path-field shapes | Exits 1 before creating the registry |
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
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-26 | v4: Add safe name/key serialization, config preflight, truthful no-op/failure behavior, and structured output (#440) |
