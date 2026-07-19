---
change: CHG-0057-add-a-native-migration-path-for-5-0-1-era-change-ledgers-that-backfills-the-5-1
artifact: requirements
---

# Requirements

SpecSync SHALL provide a native, idempotent migration that backfills 5.1 reopening digest
fields on 5.0.1-era change ledgers with a verification pass before any write.

- `stale` always reproduces the embedded prior verification's acceptance-input digest.
- `current` comes from the superseding verification's signed digest, else a live recomputation.
- Records already carrying both fields are never modified; re-running is a no-op.
- A reopening that cannot be repaired deterministically fails without mutating its ledger.
- Repaired ledgers re-parse and re-validate before the write lands.
- `check` on an un-migrated ledger prints the `specsync migrate 5.0` remediation, not a raw
  serde error.
