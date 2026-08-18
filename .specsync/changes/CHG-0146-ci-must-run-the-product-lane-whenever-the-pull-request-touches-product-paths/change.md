---
id: CHG-0146-ci-must-run-the-product-lane-whenever-the-pull-request-touches-product-paths
state: implementing
type: bug_fix
base_commit: 2d60b46a5e4b245e2abdfa6a323fc1109d6416f1
---

# CI must run the product lane whenever the pull request touches product paths

## Intent

CI must run the product lane whenever the pull request touches product paths

## Affected Canonical Specs

- None

## Acceptance Criteria

- a pull request whose diff touches product paths runs the product lane even when its tip commit is a lifecycle archive move; a pull request that is genuinely archive-only still narrows the lane and skips the product jobs; the selection rule lives in a script with tests rather than in inline workflow bash; the tests fail against the previous unconditional override

## No-spec Rationale

the tip-only classification overrode the whole-PR one unconditionally, and change ship always makes an archive commit last, so every lifecycle pull request skipped test/fmt/coverage/audit/spec-check while the required aggregate reported green
