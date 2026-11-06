---
change: drop-the-ci-yml-path-filters-so-every-pull-request-starts-ci-and-classify-alone-picks-its-lane
artifact: design
---

# Design

There are two decisions in CI today, and this change leaves one of them.

| Decision | Before | After |
|---|---|---|
| Does CI start? | Only if a changed path matches one of 27 `paths:` globs (identical for `pull_request` and `push`) | Always, for every pull request to main and every push to main |
| Which jobs run? | `classify-ci-paths.sh`: named paths pick the site, vscode, specs/lifecycle, archive or review lanes; everything else goes to the full lane | Unchanged |

Removing the filter can only add runs; it never removes one. A pull request the filter used to skip now starts, and classify gives it the full lane. That is the lane classify already gave such paths whenever they shared a PR with a listed path.

The test does not copy either side's rules. It reads `ci.yml` with Ruby's Psych, as the other workflow tests do. It asks the real `classify-ci-paths.sh` about the two change sets #941 exposed. It then simulates the resulting lane with `test-required-ci-gate.py`'s own model (`load_workflow`, `Scenario`, `simulate`), which runs the gate's real shell steps.
