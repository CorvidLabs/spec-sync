---
change: CHG-0057-add-a-native-migration-path-for-5-0-1-era-change-ledgers-that-backfills-the-5-1
artifact: research
---

# Research

`reopened_change_preserves_sequence_history` shows exactly what the backfilled fields must
satisfy: `stale` must equal `prior_verification.acceptance_input_digest`, `stale` and `current`
must differ, the superseded approval must authenticate against `closing_digest`, and the
superseded approval must be present in the ledger. The `current` value itself is not otherwise
re-derived by validators, so the superseding verification's signed digest is a sound recorded
proof for historical reopenings — the same assignment the CorvidLabs rollout script used across
22 repos.

The existing `cmd_migrate` pipeline is keyed to the v3→v4 `.specsync/version` stamp and never
touches change ledgers, so the backfill lands as a separate source-family mode on the same
command rather than as another migration step. `load_approvals` is the single parse chokepoint
for `approvals.json`, which makes it the right place to attach the remediation hint.

5.0.1-era reopenings embed a complete `prior_verification` (including its acceptance-input
digest), so the repair is fully deterministic without consulting git history; archived changes
are covered by the same walk as active ones.
