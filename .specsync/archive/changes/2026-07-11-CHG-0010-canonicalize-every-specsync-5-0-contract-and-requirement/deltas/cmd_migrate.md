## ADDED

### REQUIREMENT REQ-cmd-migrate-001

The migration command SHALL upgrade supported 3.x layouts to canonical 4.0 metadata without silent data loss and with idempotent preview, backup, and recovery behavior.

Acceptance Criteria
- `specsync migrate` on a 3.x project produces a valid 4.0.0 structure with all files in `.specsync/`
- Running on an already-migrated project exits 0 with no changes
- `--dry-run` shows every file move, directory creation, and frontmatter edit without writing
- All `specsync check` validations pass after migration
- Lifecycle history is preserved verbatim (no reordering, no data loss)
- Backup is created by default in `.specsync/backup-3x/` with manifest
- Clear error messages for every failure mode
- JSON output mode produces structured migration report

## MODIFIED

### SPEC SECTION Public API

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_migrate` | `root: &Path, format: OutputFormat, dry_run: bool, no_backup: bool` | `()` | Main migrate command: detect version, run preflight checks, execute migration steps, validate result |

## ADDED

### SPEC SECTION Internal Architecture

Migration uses a step-based pipeline:

| Struct/Enum | Description |
|-------------|-------------|
| `MigrationStep` | A named step with `check` (idempotency detection) and `apply` functions |
| `StepStatus` | `Done`, `Pending`, or `Partial` — returned by each step's check |
| `MigrationContext` | Shared state: root path, config, discovered specs, dry_run flag |
| `MigrationReport` | Accumulates results: steps completed, files moved, specs updated, warnings |

### SPEC SECTION Migration Steps

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

## MODIFIED

### SPEC SECTION Dependencies

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
