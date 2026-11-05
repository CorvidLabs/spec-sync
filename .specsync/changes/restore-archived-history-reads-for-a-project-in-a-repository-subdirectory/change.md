---
id: restore-archived-history-reads-for-a-project-in-a-repository-subdirectory
state: implementing
type: bug_fix
base_commit: 40dcf33a6b0f0b55dcb7f17e8e98be9d55fa2c56
---

# Restore archived history reads for a project in a repository subdirectory

## Intent

restore archived history reads for a project in a repository subdirectory

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- A project rooted in a subdirectory of its Git repository refuses a workflow-version downgrade that a later commit restored, both on a live read and from the warm read cache.
- That same project still follows an earlier dated archive path, so a downgrade that exists only on that path is refused after reopen and rearchive.

## No-spec Rationale

Not applicable
