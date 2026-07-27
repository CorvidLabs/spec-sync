---
module: cmd_init
version: 9
status: stable
files:
  - src/commands/init.rs
db_tables: []
implements: [440]
tracks: []
depends_on:
  - specs/config/config.spec.md
  - specs/types/types.spec.md
---

# Cmd Init

## Purpose

Implements `specsync init` and `init --repair`. Creates the 5.0 `.specsync/` layout with truthful source detection, canonical TOML configuration, SDD policy, version stamp, local-state ignore rules, lifecycle/change/archive directories, structured outcomes, and optional guided agent/change bootstrap.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_init` | `root: &Path, repair: bool, format: OutputFormat` | `()` | Create, inspect, or repair the 5.0 `.specsync/` layout and render a format-aware outcome |
| `ensure_hashes_gitignored` | `root: &Path` | `Result<bool, String>` | Add `.specsync/hashes.json` to the root `.gitignore` (idempotent); returns `Ok(true)` if the entry was added, `Ok(false)` if already present, `Err` if the write fails |

## Invariants

1. Auto-detects source directories via `config::detect_source_dirs_with_confidence()` and never labels the fallback as detected.
2. Never overwrites an existing current or legacy configuration; legacy configurations receive a migration hint.
3. Writes the 5.0 policy, version, and layout deterministically without blocking in non-interactive environments.
4. Local hash cache, lifecycle lock, and transaction journal files are ignored and never treated as portable project state.
5. Re-running initialization is idempotent.
6. `--repair` validates the existing config before mutation, restores only missing support artifacts, and never rewrites config or specs.
7. Nested initialization is rejected when an initialized ancestor exists.
8. Predictable blocking paths are preflighted before layout creation so failures do not leave partial directories.
9. JSON, Markdown/GitHub, table, and CSV formats emit a single truthful command outcome; structured modes never launch interactive bootstrap prompts.

## Behavioral Examples

### Scenario: First init

- **Given** no config exists
- **When** `cmd_init(root, false, Text)` runs
- **Then** creates `.specsync/config.toml`, `.specsync/version`, `.specsync/.gitignore`, and the `lifecycle/`, `changes/`, `archive/` directories

### Scenario: Config exists

- **Given** `.specsync/config.toml` (or a legacy config) already exists
- **When** `cmd_init(root, false, Text)` runs
- **Then** prints message and returns without changes

### Scenario: Repair partial layout

- **Given** a valid config with missing version, local ignore, policy, or lifecycle directories
- **When** `cmd_init(root, true, Json)` runs
- **Then** restores only missing support artifacts, reports each restored path, and leaves config/spec bytes unchanged

## Error Cases

| Condition | Behavior |
|-----------|----------|
| File write fails | Exits 1 |
| No source dirs detected | Creates TOML config with `source_dirs = ["src"]` and explicitly reports that it is a fallback |
| Existing config is malformed or has a wrong known path-field type | Exits 1 before repair writes |
| Initialized ancestor exists | Exits 1 and reports the ancestor without creating nested metadata |
| Expected support directory is blocked by a file/symlink | Exits 1 before any layout write |

## Dependencies

**Consumes**

| Module | What is used |
|--------|-------------|
| config | `detect_source_dirs_with_confidence`, `validate_config_file`, `config_to_toml` |

**Consumed By**

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync init` |

**Frontmatter Synchronization**

Implementation SHALL add these canonical dependency specs to `depends_on`: `specs/types/types.spec.md`. This YAML frontmatter update is an explicit implementation edit because semantic section deltas do not apply frontmatter.

## Change Log

| Date | Change |
|------|--------|
| 2026-07-10 | v3: initialize 5.0 SDD policy/archive and offer guided agent plus first-change bootstrap |
| 2026-04-09 | Initial spec |
| 2026-06-11 | v2: Init the v4 `.specsync/` layout instead of the legacy `specsync.json` so a fresh project never sees the migration nag |
| 2026-07-11 | CHG-0002-harden-specsync-5-0-lifecycle-safety-and-release-validation: Harden SpecSync 5.0 lifecycle safety and release validation |
| 2026-07-11 | CHG-0003-finalize-specsync-5-0-release-consistency-and-parallel-validation: Finalize SpecSync 5.0 release consistency and parallel validation |
| 2026-07-11 | CHG-0004-close-final-pr-review-gaps-in-5-0-lifecycle-enforcement: Close final PR review gaps in 5.0 lifecycle enforcement |
| 2026-07-11 | CHG-0006-close-final-specsync-5-0-evidence-monorepo-bootstrap-reporting-and-import-re: Close final SpecSync 5.0 evidence, monorepo, bootstrap, reporting, and import review gaps |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-26 | v9: Add truthful detection, audited repair semantics, ancestor/preflight safety, and structured outputs (#440) |
