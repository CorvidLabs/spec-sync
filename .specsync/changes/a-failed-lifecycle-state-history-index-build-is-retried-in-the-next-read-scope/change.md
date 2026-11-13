---
id: a-failed-lifecycle-state-history-index-build-is-retried-in-the-next-read-scope
state: draft
type: bug_fix
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# A failed lifecycle-state history index build is retried in the next read scope

## Intent

A failed lifecycle-state history index build is retried in the next read scope

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- after one failed lifecycle-state history build, the same read scope does not retry it and still falls back per archive, and the next read scope at the same HEAD builds the index; a built index is still reused across scopes at the same HEAD; verdicts are unchanged

## No-spec Rationale

REQ-change-140 builds one lifecycle-state history index per command and falls back per archive only when it cannot be built; a failed build was remembered per thread, so every later read scope at the same HEAD skipped the build (#982)
