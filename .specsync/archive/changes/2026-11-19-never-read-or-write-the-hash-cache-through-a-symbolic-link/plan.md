---
change: never-read-or-write-the-hash-cache-through-a-symbolic-link
artifact: plan
---

# Plan

## Approach

Write the failing tests first on `3bebe8b0`, then make `HashCache` refuse a linked or non-regular cache location and write through a new file renamed in a `.specsync` directory handle. `check` says once that the cache was not used and changes nothing else.

## Out of scope

- Other files written under `.specsync` (change records, locks, archives) and other commands.
- What the cache hashes: a spec's `files:` entries are still read where they point.
- A warning for save failures other than a refused location.
- Publishing the advisory and releasing. Both need the human's decision.

## Steps

1. Unit tests in `src/hash_cache.rs` and integration tests in `tests/integration/` for a linked `hashes.json` (existing target and dangling), a linked `.specsync`, a directory at `hashes.json`, and the rename. Run them on the base and keep the failures as evidence.
2. Implement the directory handle, `save`, `load` and `refusal`.
3. Print the warning in `check` and skip the save.
4. Update the `hash_cache` and `cmd_check` specs, companions and lessons, `CHANGELOG.md`, `site/src/content/docs/cli.md` and `docs/HLD.md`.
5. `change check --commit`, then the verify lane locally, since the private fork runs no CI.

## Risks

- A project that links `.specsync` or `hashes.json` on purpose loses the cache and gets a warning on each run. The check still runs and gives the same verdict.
- A writable `hashes.json` in a read-only `.specsync` can no longer be saved, because the new file is created beside it. The check is slower, not wrong.
- A killed process can leave one `.hashes.json.*.tmp` file in `.specsync`. It is never read.

## Constraints consulted

- `specs/hash_cache/hash_cache.3md`, "Conservative on uncertainty": an unusable cache may only cause more validation. A refused location reads as an empty cache.
- `specs/cmd_check/cmd_check.3md`: the cache is saved only with zero errors, and a skip is never a claim that nothing was wrong. The cache not being used is said on stderr and changes no verdict.
