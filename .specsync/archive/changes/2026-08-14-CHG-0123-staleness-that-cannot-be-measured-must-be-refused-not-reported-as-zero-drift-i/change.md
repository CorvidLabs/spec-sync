---
id: CHG-0123-staleness-that-cannot-be-measured-must-be-refused-not-reported-as-zero-drift-i
state: archived
type: bug_fix
base_commit: f26768913eeef4dac2d925ad85daddfa22a054ce
---

# Staleness that cannot be measured must be refused, not reported as zero drift, in every reader: report, check --stale, the lifecycle no_stale guard, and the score freshness dimension

## Intent

Staleness that cannot be measured must be refused, not reported as zero drift, in every reader: report, check --stale, the lifecycle no_stale guard, and the score freshness dimension

## Affected Canonical Specs

- `git_utils`
- `cmd_report`
- `cmd_check`
- `cmd_stale`
- `cmd_lifecycle`
- `scoring`
- `mcp`

## Acceptance Criteria

- A tree whose git history cannot answer the staleness question is refused by every reader rather than reported as zero drift. Both unmeasurable states are covered: no repository at all, and a repository with an unborn HEAD. 'report' and 'check --stale' exit non-zero naming which of the two it is, matching what 'stale' already did. The lifecycle no_stale guard refuses to promote on an unasked question. The score freshness dimension withholds its points rather than awarding them, so removing git can no longer raise a grade. A healthy repository reports exactly what it reported before, in every command and format.

## No-spec Rationale

Not applicable
