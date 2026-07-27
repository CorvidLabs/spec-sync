---
id: CHG-0067-fix-issue-467-by-deduplicating-identical-stage-zero-entries-from-overlapping-gi
state: archived
type: bug_fix
base_commit: e27cba1cdcb02b36ba7e4094b5ec5369a675b47a
---

# Fix issue #467 by deduplicating identical stage-zero entries from overlapping Git pathspec batches while rejecting conflicting mode or object observations

## Intent

Fix issue #467 by deduplicating identical stage-zero entries from overlapping Git pathspec batches while rejecting conflicting mode or object observations

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- Identical stage-zero entries returned through overlapping bounded pathspec batches are accepted and represented exactly once.
- A repeated path with a different Git mode fails closed without replacing the first observed entry.
- A repeated path with a different Git object ID fails closed without replacing the first observed entry.
- A lifecycle delivery scope containing a parent directory and enough exact tracked children to cross the pathspec batch boundary completes Git candidate inspection successfully.
- Existing deterministic output bounds, unresolved-stage rejection, and out-of-scope path rejection remain unchanged.

## No-spec Rationale

Not applicable
