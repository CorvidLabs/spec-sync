---
id: merge-two-finalized-changes-to-one-module-without-a-spec-conflict
state: archived
type: feature
base_commit: 40dcf33a6b0f0b55dcb7f17e8e98be9d55fa2c56
---

# Merge two finalized changes to one module without a spec conflict

## Intent

Merge two finalized changes to one module without a spec conflict

## Affected Canonical Specs

- `merge`
- `cmd_merge`
- `validator`
- `change`
- `cli`
- `cli_args`

## Acceptance Criteria

- MERGE-1: Two independent changes to the same module, each finalized on its own branch, merge cleanly in either order with no conflict on that module's .spec.md (version:, change-log row, files:) or on requirements.md.
- MERGE-2: A fixture with two such branches proves it with git merge-tree (failing first, then green).
- MERGE-3: After a merge that leaves the same requirement ID twice, both check and change audit refuse and name the duplicate ID.
- MERGE-4: Existing strictness stays: no looser gates, timeouts, or weakened assertions.

## No-spec Rationale

Not applicable
