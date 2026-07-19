---
change: CHG-0057-add-a-native-migration-path-for-5-0-1-era-change-ledgers-that-backfills-the-5-1
artifact: tasks
---

# Tasks

- [x] Implement the `migrate 5.0` ledger backfill with verified, idempotent, dry-run-aware
  writes across active and archived changes.
- [x] Add the actionable migrate hint to `load_approvals` parse failures.
- [x] Add regression coverage for backfill, idempotency, dry-run, unrepairable records, and the
  check hint.
- [x] Add and map canonical requirements (change, cmd_migrate, cli_args) and extend the
  canonical Invariants section.
- [x] Run pre-acceptance formatting, lint, unit/integration tests, and release validators.
