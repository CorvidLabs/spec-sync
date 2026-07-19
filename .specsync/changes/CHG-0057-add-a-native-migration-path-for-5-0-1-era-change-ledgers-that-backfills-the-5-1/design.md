---
change: CHG-0057-add-a-native-migration-path-for-5-0-1-era-change-ledgers-that-backfills-the-5-1
artifact: design
---

# Design

Extend the existing `migrate` command with an optional source-family positional:
`specsync migrate 5.0` runs the ledger backfill; bare `migrate` keeps the current v3→v4 flow
unchanged. The backfill is a no-op on ledgers that are already current, so it is safe to run
repeatedly (idempotent) and supports `--dry-run`.

For every active and archived change, each reopening missing the digest fields is repaired
deterministically: `stale` comes from the reopening's embedded `prior_verification`
acceptance-input digest (the only honest source — the field must reproduce it), and `current`
comes from the superseding verification's signed digest when a later verification exists, else
from a live recomputation over the current inputs. A reopening whose `stale` is absent or whose
repair yields `stale == current` cannot prove the recorded drift and is reported as a failure
without mutating that change's ledger; other repairable changes still migrate.

After repair, each ledger is re-parsed under the 5.1 schema and the repaired reopenings are
re-validated for field consistency before the write lands, so a failed verification leaves the
file untouched. Writes use the same prepared-file path as other lifecycle mutations. In
`load_approvals`, a parse failure naming the missing digest fields gains the remediation hint
(`run specsync migrate 5.0`), so every caller surfaces the actionable message instead of the raw
serde error.
