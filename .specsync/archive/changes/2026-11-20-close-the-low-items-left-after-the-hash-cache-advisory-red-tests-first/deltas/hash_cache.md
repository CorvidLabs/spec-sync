## MODIFIED
### REQUIREMENT REQ-hash-cache-007

The hash cache SHALL never be read or written through a symbolic link, and SHALL be written only inside the project's own `.specsync` directory. `HashCache::save` SHALL refuse, writing nothing, when `.specsync` is a symbolic link or not a directory, or when `.specsync/hashes.json` is a symbolic link or anything other than a regular file. Otherwise it SHALL write the cache to a new file in `.specsync` and rename that file over `.specsync/hashes.json`. `HashCache::load` SHALL return an empty cache for the same locations, without reading what a link points to. `HashCache::refusal` SHALL say which path is refused and what it is, and SHALL return nothing for a usable or absent cache.
`save` SHALL also refuse, writing nothing and leaving the cache already there, a cache whose JSON is larger than the bound `load` reads (64 MiB).

Acceptance Criteria
- With `.specsync/hashes.json` a symbolic link to an existing file outside the project, `save` returns an error, the link is still a link and that file's bytes are unchanged; `load` returns an empty cache even when that file holds a valid cache.
- With that link dangling, `save` returns an error and nothing is created at its target.
- With `.specsync` a symbolic link to a directory outside the project, `save` returns an error and that directory gains no entry; `load` returns an empty cache even when that directory holds a valid `hashes.json`.
- With `.specsync/hashes.json` a directory, `save` returns an error and `load` returns an empty cache.
- With a regular `.specsync/hashes.json`, `save` replaces it by rename: a second hard link to the old file keeps the old bytes, the new cache round-trips through `load`, and `.specsync` holds no file besides `hashes.json`.
- `refusal` names `.specsync/hashes.json` or `.specsync` in each refused case and returns nothing for a regular or absent cache.
- A cache whose JSON is over 64 MiB is refused, and the cache already there is unchanged.
