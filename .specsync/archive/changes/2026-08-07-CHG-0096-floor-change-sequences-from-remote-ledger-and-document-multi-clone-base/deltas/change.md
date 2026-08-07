## ADDED

### REQUIREMENT REQ-change-055

Change sequence allocation SHALL floor on the highest sequence observed locally (active, archive, local ledger) and, when available, the remote default-branch `.specsync/change-sequence.json` high-water. Concurrent multi-clone fleets MAY set `SPECSYNC_SEQUENCE_BASE` to disjoint ranges so agents that cannot see each other do not mint the same numeric CHG prefix.

Acceptance Criteria

- When `origin/HEAD` (or `origin/main` / `origin/master`) contains a schema-v1 sequence ledger, `change new` allocates above that sequence even if the local ledger file is missing or lower.
- `SPECSYNC_SEQUENCE_BASE=N` makes the next allocated sequence at least `N`.
- Simultaneous clones without BASE or a fetched remote high-water may still collide; post-merge sequence validation continues to fail closed on unacknowledged duplicates.
