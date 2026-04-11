---
spec: cmd_migrate.spec.md
---

## Tasks

### Done

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

### In Progress

- [ ] Test on real 3.x project (dogfood on CorvidLabs repos)

### Gaps

- Consider: migration for projects using spec-sync as a dependency (cross-project registries)
