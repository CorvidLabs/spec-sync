---
spec: cmd_migrate.spec.md
---

## Automated Testing

| Test File | Type | What It Covers |
|-----------|------|----------------|
| `src/commands/migrate.rs` inline tests | Unit | Validate Cmd Migrate behavior close to implementation, especially `cmd_migrate`, `()`, `MigrationStep`, `StepStatus`, `MigrationContext`, `MigrationReport`, `detect_version`, `create_backup` |
| `tests/integration.rs` | Integration | Exercise Cmd Migrate through project workflows and spec validation fixtures |

## Manual Testing

- [ ] Run `fledge spec check --strict` after changing Cmd Migrate contracts or source files.
- [ ] Run `fledge run test` and confirm Cmd Migrate unit/integration coverage still passes.
- [ ] Smoke-test `cargo run -- migrate --help` or the nearest CLI path that routes through this module.

## Edge Cases & Boundary Conditions

| Scenario | Expected Behavior |
|----------|-------------------|
| No `specsync.json` found and no `.specsync/config.json` | Error: "No spec-sync project found. Run `specsync init` first" |
| Permission denied writing to `.specsync/` | Error with path and suggestion to check permissions |
| Spec file with malformed frontmatter | Warning: skip that spec's lifecycle extraction, continue with others, report in summary |
| Disk full during backup | Error: "Backup failed — original files untouched. Free disk space and retry" |
