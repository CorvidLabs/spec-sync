## ADDED

### REQUIREMENT REQ-cmd-check-018

When the hash cache location is refused (REQ-hash-cache-007), `check` SHALL print one warning on stderr that names the refused path, SHALL validate every spec as it does with no cache, SHALL NOT read, create or write anything through that location, and SHALL keep the stdout output, verdict and exit code it has with no cache.

Acceptance Criteria
- With `.specsync/hashes.json` a symbolic link to a file outside the project, `specsync check`, `check --force`, `check --strict` and `check --format json` each exit as they do without the link, print `hash cache not used` once on stderr, and leave the link and the bytes of its target unchanged.
- With that link dangling, nothing is created at its target.
- With `.specsync` a symbolic link to a directory outside the project, `specsync check` prints the same warning and that directory gains no entry.
- A project whose cache is a regular file or absent prints no such warning.
