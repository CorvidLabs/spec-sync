---
module: cmd_init
version: 3
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

Implements the `specsync init` command. Creates the v4 `.specsync/` layout — `config.toml` with auto-detected source directories, a `version` stamp, `.gitignore`, and the `lifecycle/`, `changes/`, and `archive/` state directories — matching what `specsync migrate` produces.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_init` | `root: &Path` | `()` | Create the v4 `.specsync/` layout with auto-detected source dirs |
| `ensure_hashes_gitignored` | `root: &Path` | `Result<bool, String>` | Add `.specsync/hashes.json` to the root `.gitignore` (idempotent); returns `Ok(true)` if the entry was added, `Ok(false)` if already present, `Err` if the write fails |

## Invariants

1. Auto-detects source directories via `config::detect_source_dirs()`
2. Will not overwrite an existing config (v4 `.specsync/config.toml`/`config.json` or legacy `specsync.json`/`.specsync.toml`); legacy configs get a `specsync migrate` hint
3. Writes default config with detected dirs and standard required sections via `config::config_to_toml()`
4. A fresh init never triggers the legacy 3.x layout migration nag — `.specsync/version` is stamped with 4.0.0

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
| No source dirs detected | Creates config with empty `sourceDirs` |

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
