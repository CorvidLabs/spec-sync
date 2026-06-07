---
spec: rehash.spec.md
---

## Key Decisions

- **Full rebuild only**: `rehash` regenerates the cache from discovered specs rather than applying incremental updates.
- **Local cache artifact**: The command writes `.specsync/hashes.json`, which is expected to remain gitignored.
- **Failure is explicit**: Cache save failures exit non-zero because stale cache state can hide validation work.

## Files to Read First

- `src/commands/rehash.rs` — Command entry point.
- `src/hash_cache.rs` — Cache structure, companion discovery, and save behavior.

## Current Status

Stable. The command is used to refresh local hash state after branch switches, pulls, or cache deletion.

## Notes

- Keep this command deterministic: it should reflect current files on disk and avoid network or git dependencies.
