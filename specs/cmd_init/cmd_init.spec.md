---
module: cmd_init
version: 5
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

Implements `specsync init`. Creates the 5.0 `.specsync/` layout with detected source directories, canonical TOML configuration, SDD policy, version stamp, local-state ignore rules, lifecycle/change/archive directories, and optional guided agent/change bootstrap.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_init` | `root: &Path` | `()` | Create the 5.0 `.specsync/` layout, TOML config, SDD policy, and detected verification command |
| `ensure_hashes_gitignored` | `root: &Path` | `Result<bool, String>` | Add `.specsync/hashes.json` to the root `.gitignore` (idempotent); returns `Ok(true)` if the entry was added, `Ok(false)` if already present, `Err` if the write fails |

## Invariants

1. Auto-detects source directories via `config::detect_source_dirs()`.
2. Never overwrites an existing current or legacy configuration; legacy configurations receive a migration hint.
3. Writes the 5.0 policy, version, and layout deterministically without blocking in non-interactive environments.
4. Local hash cache, lifecycle lock, and transaction journal files are ignored and never treated as portable project state.
5. Re-running initialization is idempotent.

## Behavioral Examples

### Scenario: First init

- **Given** no config exists
- **When** `cmd_init(root)` runs
- **Then** creates `.specsync/config.toml`, `.specsync/version`, `.specsync/.gitignore`, and the `lifecycle/`, `changes/`, `archive/` directories

### Scenario: Config exists

- **Given** `.specsync/config.toml` (or a legacy config) already exists
- **When** `cmd_init(root)` runs
- **Then** prints message and returns without changes

## Error Cases

| Condition | Behavior |
|-----------|----------|
| File write fails | Exits 1 |
| No source dirs detected | Creates TOML config with `source_dirs = ["src"]` fallback |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| config | `detect_source_dirs`, `config_to_toml` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync init` |

## Change Log

| Date | Change |
|------|--------|
| 2026-07-10 | v3: initialize 5.0 SDD policy/archive and offer guided agent plus first-change bootstrap |
| 2026-04-09 | Initial spec |
| 2026-06-11 | v2: Init the v4 `.specsync/` layout instead of the legacy `specsync.json` so a fresh project never sees the migration nag |
| 2026-07-11 | CHG-0002-harden-specsync-5-0-lifecycle-safety-and-release-validation: Harden SpecSync 5.0 lifecycle safety and release validation |
| 2026-07-11 | CHG-0003-finalize-specsync-5-0-release-consistency-and-parallel-validation: Finalize SpecSync 5.0 release consistency and parallel validation |
