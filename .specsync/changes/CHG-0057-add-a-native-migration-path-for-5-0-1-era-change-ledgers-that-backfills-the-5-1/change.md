---
id: CHG-0057-add-a-native-migration-path-for-5-0-1-era-change-ledgers-that-backfills-the-5-1
state: accepted
type: feature
base_commit: 16fc94b95ce39dcdcdf9019252e6ad7eb733deef
---

# Add a native migration path for 5.0.1-era change ledgers that backfills the 5.1 reopening stale and current acceptance-input digest fields idempotently with a closing-digest verification pass, and surfaces an actionable migrate hint when check encounters the 5.0.1 reopening schema

## Intent

Add a native migration path for 5.0.1-era change ledgers that backfills the 5.1 reopening stale and current acceptance-input digest fields idempotently with a closing-digest verification pass, and surfaces an actionable migrate hint when check encounters the 5.0.1 reopening schema

## Affected Canonical Specs

- `change`
- `cmd_migrate`
- `cli_args`

## Acceptance Criteria

- specsync migrate 5.0 backfills missing stale/current acceptance-input digest fields on every 5.0.1-era reopening across active and archived change ledgers; the backfill is idempotent (re-running changes nothing), verifies closing digests after writing, supports --dry-run, and fails without mutation when a reopening cannot be repaired deterministically; a ledger containing 5.0.1-era reopenings parses and validates after migration; specsync check on an un-migrated 5.0.1 ledger prints an actionable migrate hint instead of a raw serde missing-field error; regression tests cover backfill, idempotency, dry-run, verification failure, and the check hint.

## No-spec Rationale

Not applicable
