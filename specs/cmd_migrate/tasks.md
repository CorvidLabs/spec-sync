---
spec: cmd_migrate.spec.md
---

## Tasks

#

## Post-5.0 Test Debt

- [ ] Add unit tests for the remaining `apply_*` steps (config/registry relocation, lifecycle extraction, frontmatter cleanup)

## Done

- [x] Add `Migrate` variant to CLI Command enum in `cli.rs`
- [x] Create `src/commands/migrate.rs` with step-based architecture
- [x] Implement `MigrationStep`, `StepStatus`, `MigrationContext`, `MigrationReport` types
- [x] Implement step 1: version detection (3.x vs 4.0)
- [x] Implement step 2: backup creation with manifest.json
- [x] Implement step 3: directory structure creation
- [x] Implement step 4: config relocation (JSON → TOML conversion)
- [x] Implement step 5: registry relocation
- [x] Implement step 6: lifecycle history extraction from frontmatter
- [x] Implement step 7: frontmatter cleanup (remove lifecycle_log)
- [x] Implement step 8: .gitignore creation
- [x] Implement step 9: cross-project reference scanning
- [x] Implement step 10: version stamp
- [x] Wire up `cmd_migrate` in `main.rs`
- [x] Add `--dry-run` and `--no-backup` flags
- [x] JSON output mode for migration report
- [x] TOML config format (decided: TOML, matching registry format)
- [x] Auto-detection of 3.x layout with migration suggestion in `specsync check`
- [x] Integration tests for the full v3→v4 flow (`migrate_full_v3_to_v4`, dry-run, idempotency, partial recovery, JSON, `--no-backup`, no-project)
- [x] Unit tests for migration-step application (`apply_create_directories_creates_v4_layout`, `apply_create_directories_is_idempotent`)
- [x] Add `migrate 5.0` source-family mode with idempotent, verified reopening digest backfill

### Open

- [x] Test on real 3.x project (dogfood on CorvidLabs repos) — Evidence: repository migration commit `68829c5`.

### Gaps

- Most `apply_*` steps are exercised only through the end-to-end integration flow; only `apply_create_directories` has dedicated unit coverage.
- Consider: migration for projects using spec-sync as a dependency (cross-project registries)

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
