---
id: stop-restoring-a-build-cache-in-the-job-that-produces-the-released-binaries-because-save-if-false-proves-the-job-cannot
state: implementing
type: operations
base_commit: db1f4ac95d0a81eecb1777d351a52222fb1aa75f
---

# Stop restoring a build cache in the job that produces the released binaries, because save-if false proves the job cannot poison the cache and not that its output is trustworthy

## Intent

Stop restoring a build cache in the job that produces the released binaries, because save-if false proves the job cannot poison the cache and not that its output is trustworthy

## Affected Canonical Specs

- `github`

## Acceptance Criteria

- The release build job contains no caching step and a comment stating why it must never gain one; the qualify job's cache is unchanged and the reason for leaving it is recorded; release.yml still parses; and CHANGELOG.md states what save-if false does and does not establish, and that two sibling CodeQL alerts were dismissed as already mitigated while this one was not.

## No-spec Rationale

The github module owns .github/workflows, so ownership is declared with --spec; the edit removes a caching step and adds prose, so no canonical spec text moves.
