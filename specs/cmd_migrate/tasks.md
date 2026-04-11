---
spec: cmd_migrate.spec.md
---

## Tasks

### Done

(none yet)

### In Progress

- [ ] Add `Migrate` variant to CLI Command enum in `cli.rs`
- [ ] Create `src/commands/migrate.rs` with step-based architecture
- [ ] Implement `MigrationStep`, `StepStatus`, `MigrationContext`, `MigrationReport` types
- [ ] Implement step 1: version detection (3.x vs 4.0)
- [ ] Implement step 2: backup creation with manifest.json
- [ ] Implement step 3: directory structure creation
- [ ] Implement step 4: config relocation
- [ ] Implement step 5: registry relocation
- [ ] Implement step 6: lifecycle history extraction from frontmatter
- [ ] Implement step 7: frontmatter cleanup (remove lifecycle_log)
- [ ] Implement step 8: hash cache rebuild
- [ ] Implement step 9: .gitignore creation
- [ ] Implement step 10: version stamp
- [ ] Implement step 11: post-migration validation
- [ ] Wire up `cmd_migrate` in `main.rs`
- [ ] Add `--dry-run` and `--no-backup` flags
- [ ] JSON output mode for migration report
- [ ] Test on real 3.x project (dogfood on CorvidLabs repos)

### Gaps

- Need to decide: should config format change from JSON to TOML to match registry? Or keep JSON?
- Need to decide: should `specsync check` auto-detect 3.x layout and suggest migration?
- Consider: migration for projects using spec-sync as a dependency (cross-project registries)
