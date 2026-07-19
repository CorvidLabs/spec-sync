## ADDED

### REQUIREMENT REQ-cmd-migrate-002

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

## MODIFIED

### SPEC SECTION Invariants

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
