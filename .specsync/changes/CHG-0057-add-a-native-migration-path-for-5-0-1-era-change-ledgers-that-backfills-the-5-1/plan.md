---
change: CHG-0057-add-a-native-migration-path-for-5-0-1-era-change-ledgers-that-backfills-the-5-1
artifact: plan
---

# Plan

1. Add the `migrate 5.0` source-family mode and the ledger backfill in `src/change.rs` with an
   idempotent, verified, dry-run-aware write path.
2. Surface the migrate hint from `load_approvals` parse failures naming the missing fields.
3. Cover backfill, idempotency, dry-run, unrepairable records, and the check hint with
   regression tests.
4. Add canonical requirements (change, cmd_migrate, cli_args) and extend the canonical
   Invariants section.
5. Accept the change, then run forced strict and the complete Trust lane.
