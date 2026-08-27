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
- Must use only operations that behave identically on every host a repository may be checked out
  on, macOS, Linux, and Windows included (no symlinks, no Unix-specific operations). This holds for
  Windows even though 6.0 publishes no Windows binary, because a migrated repository is read and
  re-migrated in clones on other hosts
- Must handle large projects (100+ specs) without excessive memory usage
- Backup should be opt-out (`--no-backup`) not opt-in
- Created files and directories use the platform default permissions (`fs::create_dir_all` / `fs::write`); no explicit mode is set

## Out of Scope

- Migration from versions before 3.x
- Interactive/wizard-style migration (fully automatic, non-interactive)
- Automatic git commit of migration results (user decides when to commit)
- Downgrade from 4.0 back to 3.x (backup enables manual rollback)

### REQ-cmd-migrate-001

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

### REQ-cmd-migrate-002

The migration command SHALL provide a `5.0` source-family mode that backfills 5.1 reopening
digest fields across active and archived change ledgers with idempotent, dry-run-aware,
verification-gated writes.

Acceptance Criteria

- `specsync migrate 5.0` repairs every deterministically repairable reopening and reports
  per-change results.
- Running on an already-migrated ledger reports zero repairs and changes no bytes.
- `--dry-run` reports planned repairs without writing.
- An unrepairable reopening fails its change without mutating that ledger; other changes still
  migrate.
- The v3→v4 migration pipeline is unchanged and never runs in `5.0` mode.

### REQ-cmd-migrate-003

Migration SHALL use only filesystem operations that behave identically on every host platform a repository may be checked out on, independently of which platforms SpecSync publishes binaries for.

Acceptance Criteria
- No migration step creates a symlink. A relocated file is moved or copied, because symlinks are fragile on Windows and confuse git.
- No step depends on Unix-specific semantics such as an explicit permission mode; created files and directories take the platform default.
- The constraint continues to hold for Windows even though 6.0 publishes no Windows binary, because a migrated repository is committed once and then read, re-checked, and re-migrated in clones on other hosts.

