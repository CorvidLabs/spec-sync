---
id: CHG-0156-the-reopen-then-close-guard-must-be-pinned-by-tests-not-only-by-a-drill
state: archived
type: feature
base_commit: eb8f863af9a7e5822bf0a207f30a87302da7891a
---

# The reopen-then-close guard must be pinned by tests, not only by a drill

## Intent

the reopen-then-close guard must be pinned by tests, not only by a drill

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- Removing the archive-to-active direction from the scoped review history guard, which is the defect a reopened change could never be closed again, fails a test rather than leaving the suite green; deleting the guard entirely also fails a test, and fails a different one, so neither the fix nor the refusal it lives inside can be removed silently; the refusal of a move to any location other than a change's two homes is asserted for the first time, in both directions; and deleting committed review evidence is still refused.

## No-spec Rationale

Not applicable
