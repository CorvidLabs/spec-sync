---
change: never-read-or-write-the-hash-cache-through-a-symbolic-link
artifact: research
---

# Research

Every reader and writer of `.specsync/hashes.json` at base `3bebe8b0`:

| Place | What it does | After this change |
|---|---|---|
| `HashCache::save` (`src/hash_cache.rs`) | `create_dir_all`, then `fs::write`, both following links | Refuses a linked or non-regular location; writes a new file and renames it |
| `HashCache::load` (`src/hash_cache.rs`) | `fs::read_to_string`, following links | Reads no link and no non-regular file; returns an empty cache |
| `cmd_check` (`src/commands/check.rs`) | loads, then `let _ = cache.save(root)` after a run with no errors | Warns once when the location is refused and skips the cache |
| `cmd_rehash` (`src/commands/rehash.rs`) | saves, and already exits 1 when `save` fails | Unchanged source; the refusal reaches it as a save error |
| `tool_check` (`src/mcp/tools.rs`) | loads only; `validate_known_config_files` already rejects a linked `hashes.json` | Unchanged source; `load` is safe by itself too |
| `project_input_is_volatile`, `HASH_CACHE_PATH` (`src/change/`) | name the path to leave it out of digests and to stage it; no read or write | Unchanged |
| `init`, `migrate` | write the `.gitignore` entry for the path; no read or write of the cache | Unchanged |

The repository already confines security-sensitive file access with `cap-std` directory handles and no-follow opens (`generator`, `watch`, `validator`, `mcp`). The cache uses the same: it opens the project root as a `Dir`, opens `.specsync` without following a link, and works relative to that handle, so the write stays in that directory even if a path component is swapped while it runs.

`rename(2)` replaces a link at the destination and never writes through it, and an exclusive create never follows a link at the new name.
