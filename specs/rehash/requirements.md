---
spec: rehash.spec.md
---

## User Stories

- As a developer, I want to regenerate my hash cache after switching branches or pulling so that `specsync check` reflects the current files without needing `--force`
- As a developer who gitignores `.specsync/hashes.json`, I want a single command to rebuild it from scratch
- As a CI operator, I want a non-zero exit on cache write failure so the problem is actionable
- As a developer, I want the first unchanged check after rehash to preserve complete validation findings instead of silently treating hash-only state as a successful snapshot

## Acceptance Criteria

- `cmd_rehash` discovers all non-template specs directly and rebuilds a fresh `HashCache` from scratch (not incremental)
- The shared collected validation path records complete versioned snapshots when validation has no errors; any validation error clears all replayable snapshots so the next check validates and reports it
- Rehash stores hashes for config, recursive schema files, and `.specsyncignore` as global validation inputs
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
- Acting as a validation gate or changing exit status based on validation findings
- Reading or honoring a pre-existing cache state

### REQ-rehash-001

The rehash command SHALL rebuild the local hash cache from current canonical inputs and fail clearly when persistence fails.

Acceptance Criteria
- `cmd_rehash` loads canonical configuration and discovers non-template specs directly through validator APIs,
  without depending on the parent command registry, then rebuilds a fresh `HashCache` from scratch
- The fresh cache includes current global-input hashes and, only for an error-free validation result, complete versioned input-bound snapshots for every discovered spec
- The rebuilt cache is written to `.specsync/hashes.json`
- On success, prints a confirmation including the number of specs hashed
- On `cache.save` failure, prints an `error:` message to stderr and exits with code 1
