---
change: CHG-0057-add-a-native-migration-path-for-5-0-1-era-change-ledgers-that-backfills-the-5-1
artifact: docs
---

# Docs

Document `specsync migrate 5.0`: the native upgrade path for 5.0.1-era change ledgers. It
backfills the 5.1 reopening digest fields idempotently, verifies every repair before writing,
and is the remediation `specsync check` prints when it encounters the 5.0.1 reopening schema.
