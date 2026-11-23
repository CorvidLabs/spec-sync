---
change: never-read-or-write-the-hash-cache-through-a-symbolic-link
artifact: tasks
---

# Tasks

- [x] Failing tests on `3bebe8b0`: linked `hashes.json` (existing and dangling target), linked `.specsync`, hard link kept by the rename, `check` and `rehash` end to end
- [x] `HashCache::save`, `load` and `refusal` on a no-follow `.specsync` directory handle
- [x] `check` warns once and skips the cache when the location is refused
- [x] Specs, companions and lessons for `hash_cache` and `cmd_check`; `CHANGELOG.md`, `cli.md`, `HLD.md`

Human gates, not tasks: the scope approval before the build, the implementation review before the archive, and the decisions to publish the advisory and to release.
