---
id: watch-outcome-tests-wait-for-the-verdict-line
state: archived
type: bug_fix
base_commit: 46f55652649a72ddf5e1715938b504b96686566b
---

# Watch outcome tests wait for the verdict line

## Intent

watch outcome tests wait for the verdict line

## Affected Canonical Specs

- None

## Acceptance Criteria

- the zero-specs watch test and its control wait for the verdict line instead of a fixed 5s window

## No-spec Rationale

test-only change; no module contract moves (watch spec's missing sections are pre-existing and out of scope)
