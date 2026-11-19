---
change: never-read-or-write-the-hash-cache-through-a-symbolic-link
artifact: design
---

# Design

All of it is in `src/hash_cache.rs`, plus the warning in `src/commands/check.rs`.

- **One way into the cache directory.** A private helper opens the project root as a `cap_std::fs::Dir`, looks at `.specsync` with `symlink_metadata`, and opens it with `cap_primitives::fs::open_dir_nofollow`. A link or a non-directory is refused with an error that names `.specsync` and what it is. A missing directory is created for a save and reads as "no cache" for a load.
- **`save`.** Refuses when `.specsync/hashes.json` exists and is not a regular file. Otherwise it creates `.hashes.json.<pid>.<n>.tmp` in the directory handle with an exclusive create, writes the JSON, and renames it over `hashes.json` in the same handle. A failed write or rename removes the temporary file. The file keeps the default creation mode, as before.
- **`load`.** Opens `hashes.json` in the directory handle with no-follow and non-blocking flags, checks on the open handle that it is a regular file, then reads and parses it. Every failure is an empty cache, as before.
- **`refusal`.** `HashCache::refusal(root) -> Option<String>` returns the reason a location is refused. It is the text `check` prints.
- **`check`.** Asks for the refusal once before loading. With one, it prints `warning: hash cache not used: <reason>` on stderr and does not save. Its stdout, verdict and exit code are those of a run with no cache.
- **`rehash`.** No source change. `save` returns the refusal and the command prints it and exits 1, as it does for any save error.

Alternatives not taken:

- Path checks with `fs::symlink_metadata` followed by `tempfile` in the path: correct for a committed link, but a path component swapped between the check and the write is followed. The directory handle costs a few lines and closes that.
- Opening `hashes.json` for writing in place with no-follow: no rename, so a hard link to the file is still written through, and a reader can see a half-written cache.
- A warning for every failed save: more honest, but a change for read-only checkouts that nobody asked for here.
