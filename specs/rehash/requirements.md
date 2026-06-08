---
spec: rehash.spec.md
---

## User Stories

- As a developer, I want to regenerate my hash cache after switching branches or pulling so that `specsync check` reflects the current files without needing `--force`
- As a developer who gitignores `.specsync/hashes.json`, I want a single command to rebuild it from scratch
- As a CI operator, I want a non-zero exit on cache write failure so the problem is actionable

## Acceptance Criteria

- `cmd_rehash` discovers all specs via `load_and_discover(root, false)` and rebuilds a fresh `HashCache` from scratch (not incremental)
- The rebuilt cache is written to `.specsync/hashes.json`
- On success, prints a confirmation including the number of specs hashed
- On `cache.save` failure, prints an `error:` message to stderr and exits with code 1

## Constraints

- Must not panic on expected error conditions — print and exit
- Full rebuild only: the command does not merge with or trust any existing cache contents
- Deterministic and offline: reflects files on disk only; no git or network dependency
- Writes a local artifact (`.specsync/hashes.json`) expected to be gitignored

## Out of Scope

- Incremental / partial cache updates (that path lives in `check`, not `rehash`)
- Validating or scoring specs
- Reading or honoring a pre-existing cache state
