---
id: prefetch-workers-share-one-copy-of-the-command-s-seeded-repository-facts
state: implementing
type: bug_fix
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# Prefetch workers share one copy of the command's seeded repository facts

## Intent

Prefetch workers share one copy of the command's seeded repository facts

## Affected Canonical Specs

- `git_utils`

## Acceptance Criteria

- a repository-fact seed cloned for each prefetch worker shares one copy of the command's facts, a worker's scope still answers from them, and facts the worker learns stay its own; output is unchanged

## No-spec Rationale

REQ-git-utils-011 lets prefetch workers start from what the command already knows; each worker deep-copied every memoized Git output, so a large memoized result was copied once per worker. The seed now shares one copy; output is unchanged (#955)
