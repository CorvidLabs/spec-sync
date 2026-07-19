---
module: cmd_migrate
version: 3
status: stable
files:
  - src/commands/migrate.rs
db_tables: []
tracks: [198]
depends_on:
  - specs/cmd_lifecycle/cmd_lifecycle.spec.md
  - specs/commands/commands.spec.md
  - specs/config/config.spec.md
  - specs/hash_cache/hash_cache.spec.md
  - specs/parser/parser.spec.md
  - specs/types/types.spec.md
  - specs/validator/validator.spec.md
implements: [198]
---

# Cmd Migrate

## Purpose

Implements the `specsync migrate` command — upgrades a spec-sync project from v3.x to v4.0.0. Handles config relocation, registry relocation, lifecycle history extraction from spec frontmatter into external JSON files, directory structure creation (changes, archive), frontmatter cleanup, and backup creation. Designed to be atomic, idempotent, and safe with `--dry-run` support.

## Public API

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_migrate` | `root: &Path, format: OutputFormat, dry_run: bool, no_backup: bool` | `()` | Main migrate command: detect version, run preflight checks, execute migration steps, validate result |

## Invariants

1. Each step's `check` function is idempotent — running migrate on an already-migrated project produces zero changes and exits 0
2. `--dry-run` executes all check functions and reports what *would* change, but never writes to disk
3. Backup is created before any destructive operations (file moves, frontmatter edits)
4. If any step fails, previously completed steps are not rolled back — but the backup enables manual recovery. A clear error message identifies which step failed and how to recover
5. Lifecycle history is extracted verbatim — no data transformation, reordering, or loss
6. The old `specsync.json` and `specsync-registry.toml` are deleted after successful relocation (no symlinks — they are fragile on Windows and confuse git)
7. `.specsync/.gitignore` ships with the migration to control which files get committed vs ignored
8. Post-migration validation runs `specsync check` logic to confirm the migrated project is valid
9. Partial migration state is detected and handled — if a previous migrate crashed, re-running will skip completed steps and resume
10. The `5.0` source-family mode backfills reopening digest fields from recorded evidence only, verifies each repair before writing, and never mutates ledgers it cannot repair deterministically.

## Behavioral Examples

### Scenario: Fresh migration from 3.x

- **Given** a project with `specsync.json` in root, specs with `lifecycle_log` in frontmatter
- **When** `specsync migrate` runs
- **Then** config converts to `.specsync/config.toml`, lifecycle logs extracted to `.specsync/lifecycle/`, frontmatter cleaned, backup created in `.specsync/backup-3x/`

### Scenario: Already migrated project

- **Given** a project with `.specsync/version` containing `4.0.0`
- **When** `specsync migrate` runs
- **Then** outputs "Already at v4.0.0 — nothing to migrate" and exits 0

### Scenario: Dry run

- **Given** a 3.x project
- **When** `specsync migrate --dry-run` runs
- **Then** outputs an ordered migration plan showing what would change (files moved, frontmatter fields removed, directories created) without modifying any files

### Scenario: Partial migration recovery

- **Given** a project where a previous migrate crashed after step 4 (config relocated but registry not yet moved)
- **When** `specsync migrate` runs again
- **Then** steps 1-4 report "already done", steps 5+ execute normally

### Scenario: Spec with no lifecycle_log

- **Given** a spec that has never had a lifecycle transition
- **When** migrate runs the extract step
- **Then** no `.specsync/lifecycle/{module}.json` is created for that spec (only specs with history get files)

## Error Cases

| Condition | Behavior |
|-----------|----------|
| No `specsync.json` found and no `.specsync/config.json` | Error: "No spec-sync project found. Run `specsync init` first" |
| Permission denied writing to `.specsync/` | Error with path and suggestion to check permissions |
| Spec file with malformed frontmatter | Warning: skip that spec's lifecycle extraction, continue with others, report in summary |
| Disk full during backup | Error: "Backup failed — original files untouched. Free disk space and retry" |
| `--no-backup` with destructive steps | Proceed without backup (user opted out) |

## Dependencies

| Dependency | Why |
|------------|-----|
| `config.rs` | Load existing config via `load_config_from_path`, serialize to TOML via `config_to_toml` |
| `parser.rs` | Parse spec frontmatter to extract lifecycle_log |
| `validator.rs` | `find_spec_files` for discovering all specs during lifecycle extraction |
| `std::fs` | File I/O for moves, copies, directory creation |
| `std::time::SystemTime` | Timestamps for backup manifest and lifecycle extraction |
| `serde_json` | Serialize lifecycle history, backup manifest, migration report |
| `regex` | Parse `specsDir` from TOML config during spec discovery |

**Frontmatter Synchronization**

Implementation SHALL add these canonical dependency specs to `depends_on`: `specs/types/types.spec.md`, `specs/validator/validator.spec.md`. This YAML frontmatter update is an explicit implementation edit because semantic section deltas do not apply frontmatter.

## Change Log

| Date | Change |
|------|--------|
| 2026-04-11 | Initial spec — v3.x to v4.0.0 migration command |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-19 | CHG-0057-add-a-native-migration-path-for-5-0-1-era-change-ledgers-that-backfills-the-5-1: Add a native migration path for 5.0.1-era change ledgers that backfills the 5.1 reopening stale and current acceptance-input digest fields idempotently with a closing-digest verification pass, and surfaces an actionable migrate hint when check encounters the 5.0.1 reopening schema |

## Internal Architecture

Migration uses a step-based pipeline:

| Struct/Enum | Description |
|-------------|-------------|
| `MigrationStep` | A named step with `check` (idempotency detection) and `apply` functions |
| `StepStatus` | `Done`, `Pending`, or `Partial` — returned by each step's check |
| `MigrationContext` | Shared state: root path, config, discovered specs, dry_run flag |
| `MigrationReport` | Accumulates results: steps completed, files moved, specs updated, warnings |

## Migration Steps

The migration runs these steps in order:

| # | Step | What it does |
|---|------|-------------|
| 1 | `detect_version` | Read current config, determine if 3.x or already 4.0. Exit early if already migrated. Detects `specsync.json`, `.specsync.toml`, and `specsync-registry.toml` |
| 2 | `create_backup` | Copy config files, registry, and all spec files to `.specsync/backup-3x/` with `manifest.json` |
| 3 | `create_directories` | Create `.specsync/`, `.specsync/lifecycle/`, `.specsync/changes/`, `.specsync/archive/` |
| 4 | `relocate_config` | Convert config to TOML and write `.specsync/config.toml`, remove old `specsync.json` / `.specsync.toml` / `.specsync/config.json` |
| 5 | `relocate_registry` | Move `specsync-registry.toml` → `.specsync/registry.toml`, remove old file |
| 6 | `extract_lifecycle` | For each spec with `lifecycle_log`, extract entries into `.specsync/lifecycle/{module}.json` |
| 7 | `cleanup_frontmatter` | Remove `lifecycle_log` field from all spec frontmatter |
| 8 | `write_gitignore` | Create `.specsync/.gitignore` with sensible defaults (ignore backup-3x/, archive/, hashes.json) |
| 9 | `update_root_gitignore` | Add `.specsync/hashes.json` to the repository-root `.gitignore` |
| 10 | `scan_cross_project` | Scan specs for cross-project references and write `.specsync/cross-project-refs.json` if any found |
| 11 | `stamp_version` | Write `.specsync/version` containing `4.0.0` |

