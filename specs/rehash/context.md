---
spec: rehash.spec.md
---

## Key Decisions

- **Full rebuild only**: `rehash` regenerates the cache from discovered specs rather than applying incremental updates.
- **Local cache artifact**: The command writes `.specsync/hashes.json`, which is expected to remain gitignored.
- **Failure is explicit**: Cache save failures exit non-zero because stale cache state can hide validation work.
- **Complete rebuild**: rehash invokes the same collected validation loop as check and binds each snapshot to current global inputs and complete spec inventory.
- **Not a gate**: validation findings are stored for the next check but do not alter rehash's historical success behavior; persistence failure remains the command's blocking error.
- **Errors never become warm state**: if shared validation reports any error, rehash clears every snapshot before saving. Hashes remain refreshed, but the next check must validate and surface the errors.

## Files to Read First

- `src/commands/rehash.rs` — Command entry point.
- `src/hash_cache.rs` — Cache structure, companion discovery, and save behavior.

## Current Status

Stable. The command refreshes both hash state and replayable validation snapshots after branch switches, pulls, or cache deletion.

## Notes

- Keep this command deterministic: it should reflect current files on disk and avoid network or git dependencies.
