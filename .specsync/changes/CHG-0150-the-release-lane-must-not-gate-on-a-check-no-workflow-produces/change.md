---
id: CHG-0150-the-release-lane-must-not-gate-on-a-check-no-workflow-produces
state: implementing
type: feature
base_commit: 8fad38d405d51c8ac40f78fb18ef28ee632573a7
---

# The release lane must not gate on a check no workflow produces

## Intent

the release lane must not gate on a check no workflow produces

## Affected Canonical Specs

- None

## Acceptance Criteria

- release.yml's validate job no longer waits for a check run named 'SpecSync archive binding', so an RC tag reaches the qualify job instead of failing before it; the checks validate does keep (tag version equals the Cargo package version, the checkout is the resolved candidate, the candidate is an ancestor of origin/main) still fail closed when violated; and release.yml gains a workflow_dispatch dry-run path that exercises the lane up to but not including tag creation, so the workflow can be executed without pushing a tag.

## No-spec Rationale

release.yml is CI configuration under .github/ with no owning spec module; precedent CHG-0014
