---
id: CHG-0088-catch-verification-evidence-a-squash-merge-discarded
state: archived
type: bug_fix
base_commit: 176719bb6acde18ca96fa2eadba53fd11faa00ed
---

# Catch verification evidence a squash merge discarded

## Intent

Catch verification evidence a squash merge discarded

## Affected Canonical Specs

- `github`

## Acceptance Criteria

- the preflight fails when an active change's verification commit is absent from the repository or not an ancestor of HEAD (orphaned by squash merge, rebase, amend, or force-push)
- the error names re-check as the remedy rather than only blaming squash merges

## No-spec Rationale

Not applicable
