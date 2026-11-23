---
change: never-read-or-write-the-hash-cache-through-a-symbolic-link
artifact: requirements
---

# Requirements

Human intent CHECK-1.e: "Running the check on a repository somebody else wrote never changes a file outside that repository."

- `REQ-hash-cache-007` (added, `deltas/hash_cache.md`): the cache is never read or written through a symbolic link and is written only inside the project's own `.specsync`, by a new file renamed over `hashes.json`.
- `REQ-cmd-check-018` (added, `deltas/cmd_check.md`): with a refused cache location, `check` warns once on stderr, validates everything, and keeps its output, verdict and exit code.

`rehash` needs no new requirement: its spec already says it exits 1 when the save fails, and the refusal is a save failure.
