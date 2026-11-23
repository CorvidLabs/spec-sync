---
id: never-read-or-write-the-hash-cache-through-a-symbolic-link
state: archived
type: bug_fix
base_commit: 3bebe8b03094b2079630801584b56f94a1e460c5
---

# Never read or write the hash cache through a symbolic link

## Intent

Never read or write the hash cache through a symbolic link

## Affected Canonical Specs

- `hash_cache`
- `cmd_check`

## Acceptance Criteria

- With .specsync/hashes.json, or .specsync itself, as a symbolic link to a path outside the repository, specsync check and specsync rehash leave the link's target byte for byte unchanged and create nothing through it; check keeps its verdict and exit code and prints one stderr warning that the hash cache was not saved; rehash exits 1 and names the link; load returns an empty cache without reading through a link; a regular cache is replaced by a temp file in .specsync renamed over it; the new tests fail on 3bebe8b0 and pass with the fix

## No-spec Rationale

Not applicable
