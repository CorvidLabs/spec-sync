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

### REQ-rehash-001

The rehash command SHALL rebuild the local hash cache from current canonical inputs and fail clearly when persistence fails.

Acceptance Criteria
- `cmd_rehash` loads canonical configuration and discovers non-template specs directly through validator APIs,
  without depending on the parent command registry, then rebuilds a fresh `HashCache` from scratch
- The rebuilt cache is written to `.specsync/hashes.json`
- On success, prints a confirmation including the number of specs hashed
- On `cache.save` failure, prints an `error:` message to stderr and exits with code 1

### REQ-rehash-002

`rehash` SHALL write project files only through the confined project write functions of `util` (REQ-util-002), or through a write whose path was refused as a link just before; any other direct write SHALL say, on the write or the line above it in the same statement, why its path is not a project file (a temporary or Git directory, a test barrier, a file Git hands the command). It SHALL find spec files only through the confined, budgeted walk of `validator` (REQ-validator-050), so a `specs` link that leaves the repository, or a tree past the walk's budget, stops it with the reason instead of being walked.

Acceptance Criteria
- `tests/integration/project_writes.rs` `no_project_file_is_written_outside_the_confined_helpers` finds no unmarked direct write in this module's non-test source

### REQ-rehash-003

`rehash` SHALL save the rebuilt cache only in the project's local state folder (REQ-hash-cache-009), SHALL print the path it wrote, and SHALL exit 1 with the reason when no local state folder can be found. It SHALL never write `.specsync/hashes.json` in the work tree.

Acceptance Criteria
- After `rehash` the work tree has no `.specsync/hashes.json` and the printed path exists.

