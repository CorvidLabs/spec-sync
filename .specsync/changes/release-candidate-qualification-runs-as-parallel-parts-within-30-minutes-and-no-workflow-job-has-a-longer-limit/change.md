---
id: release-candidate-qualification-runs-as-parallel-parts-within-30-minutes-and-no-workflow-job-has-a-longer-limit
state: verifying
type: operations
base_commit: 39e7e17be6c6614f8bd49604de9c599939b9fa30
---

# Release-candidate qualification runs as parallel parts within 30 minutes and no workflow job has a longer limit

## Intent

Release-candidate qualification runs as parallel parts within 30 minutes and no workflow job has a longer limit

## Affected Canonical Specs

- `github`

## Acceptance Criteria

- A release candidate qualifies when ten parts pass, five on each of Ubuntu and macOS, each within a 30-minute limit; a platform whose parts are incomplete, failed, bound to another candidate, or whose test slices do not add up to the whole list is not qualified; no workflow job declares a limit above 30 minutes; and the lane can be rehearsed without a candidate

## No-spec Rationale

Not applicable
