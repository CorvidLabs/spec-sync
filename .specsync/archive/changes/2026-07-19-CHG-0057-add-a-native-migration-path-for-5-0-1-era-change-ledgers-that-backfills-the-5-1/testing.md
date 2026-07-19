---
change: CHG-0057-add-a-native-migration-path-for-5-0-1-era-change-ledgers-that-backfills-the-5-1
artifact: testing
---

# Testing

- `REQ-change-040`: a 5.0.1-shaped ledger (reopening without digest fields) parses and validates
  after `migrate 5.0`; `stale` equals the embedded prior-verification digest and `current` the
  superseding verification digest; a second run changes no bytes.
- `--dry-run` reports the repairs without writing; an unrepairable reopening (missing prior
  digest, or repair yielding identical digests) fails and leaves its ledger byte-identical.
- `specsync check` on an un-migrated ledger prints the `specsync migrate 5.0` hint instead of a
  bare serde missing-field error.
- `REQ-cmd-migrate-002` and `REQ-cli-args-007`: CLI integration covers the `5.0` positional,
  idempotency reporting, and `--dry-run`.
- Final `specsync check --strict --require-coverage 100 --force` and `fledge trust verify` pass.
