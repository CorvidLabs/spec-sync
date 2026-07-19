---
change: CHG-0057-add-a-native-migration-path-for-5-0-1-era-change-ledgers-that-backfills-the-5-1
artifact: context
---

# Context

GitHub issue #396. SpecSync 5.1 cannot parse 5.0.1-era change ledgers: historical reopenings in
`approvals.json` lack the required `stale_acceptance_input_digest` /
`current_acceptance_input_digest` fields, so `specsync check` fails closed with a raw serde
`missing field` error on every such record. During the 22-repo Trust 1.1.1 rollout every repo
carrying 5.0.1-era ledgers needed a field-level backfill; the fix shipped as a standalone script
in CorvidLabs/trust but belongs upstream.

The 5.1 reopen flow writes both fields at reopen time: `stale` is the prior verification's signed
acceptance-input digest and `current` is the digest of the inputs at reopen time, proving drift.
For historical reopenings the superseding re-verification's signed digest is the recorded proof
that the reopen was honored, and validation only requires the two fields to be consistent and
distinct. A deterministic, idempotent backfill can therefore repair ledgers without weakening any
check, and `check` can point at the repair instead of printing a raw parse error.
