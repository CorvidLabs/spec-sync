## MODIFIED

### REQUIREMENT REQ-cmd-check-021

`check` SHALL use only the hash cache in the project's local state folder (REQ-hash-cache-009) and SHALL never read a `.specsync/hashes.json` in the work tree or a cache under a Git directory, so a cache a repository or an archive of one ships changes nothing and prints nothing. The "Cache:" line SHALL name the cache's real path. When no local state folder can be found, or the user cache folder lies inside the project, `check` SHALL print once `warning: hash cache not used:` with the reason and validate every spec.

Acceptance Criteria
- With a forged `.specsync/hashes.json` in a tree with no Git, `check` validates the failing spec and exits 1.
- A copy of a project that ships its `.git`, cache included, validates every spec.
- With `XDG_CACHE_HOME` inside the project, `check` warns once and validates every spec.
