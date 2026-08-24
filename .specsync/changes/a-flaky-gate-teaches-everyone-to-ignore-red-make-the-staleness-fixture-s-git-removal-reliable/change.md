---
id: a-flaky-gate-teaches-everyone-to-ignore-red-make-the-staleness-fixture-s-git-removal-reliable
state: implementing
type: bug_fix
base_commit: f9d034fe1630023e082ed0088ac08862248379e0
---

# A flaky gate teaches everyone to ignore red: make the staleness fixture's git removal reliable

## Intent

A flaky gate teaches everyone to ignore red: make the staleness fixture's git removal reliable

## Affected Canonical Specs

- None

## Acceptance Criteria

- the staleness_unmeasurable module no longer fails intermittently on CI at the .git removal
- git background housekeeping is disabled in the fixture so the known concurrent writers are gone
- the removal retries on a transient failure and still panics loudly if .git genuinely cannot be removed
- no production source or spec text changes

## No-spec Rationale

Test-fixture reliability only: the fixture's .git removal races a concurrent writer on CI. No production source, canonical spec text, requirement, or behaviour changes.
