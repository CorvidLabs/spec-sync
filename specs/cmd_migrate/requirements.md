---
spec: cmd_migrate.spec.md
---

## User Stories

- As a developer using spec-sync 3.x, I want to upgrade to 4.0.0 with a single command so that I don't have to manually restructure my project
- As a CI operator, I want migration to be idempotent so that I can run it defensively in pipelines without side effects
- As a cautious user, I want `--dry-run` so that I can preview exactly what will change before committing to the migration
- As a team lead, I want lifecycle history preserved exactly so that no audit trail is lost during the upgrade

## Acceptance Criteria

- `specsync migrate` on a 3.x project produces a valid 4.0.0 structure with all files in `.specsync/`
- Running on an already-migrated project exits 0 with no changes
- `--dry-run` shows every file move, directory creation, and frontmatter edit without writing
- All `specsync check` validations pass after migration
- Lifecycle history is preserved verbatim (no reordering, no data loss)
- Backup is created by default in `.specsync/backup-3x/` with manifest
- Clear error messages for every failure mode
- JSON output mode produces structured migration report

## Constraints

- Must not panic on expected error conditions — print error and exit with non-zero code
- Must work on macOS, Linux, and Windows (no symlinks, no Unix-specific operations)
- Must handle large projects (100+ specs) without excessive memory usage
- Backup should be opt-out (`--no-backup`) not opt-in
- File permissions: directories 0755, files 0644 (explicit, not inherited)

## Out of Scope

- Migration from versions before 3.x
- Interactive/wizard-style migration (fully automatic, non-interactive)
- Automatic git commit of migration results (user decides when to commit)
- Downgrade from 4.0 back to 3.x (backup enables manual rollback)
