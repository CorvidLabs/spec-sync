---
id: CHG-0151-a-test-that-pinned-deleted-release-lane-behaviour-must-go-with-it
state: implementing
type: feature
base_commit: 64a721c9ea4c0b2cd4fbb34418f477f69162ec95
---

# A test that pinned deleted release-lane behaviour must go with it

## Intent

a test that pinned deleted release-lane behaviour must go with it

## Affected Canonical Specs

- None

## Acceptance Criteria

- test-validate-release-candidate.py no longer anchors on a release.yml line that CHG-0150 deleted, so the suite runs to completion instead of raising ValueError: substring not found; every other release.yml anchor in that file still resolves; and the suite passes at its full count.

## No-spec Rationale

.github/scripts/ is CI tooling with no owning spec module; precedent CHG-0014
