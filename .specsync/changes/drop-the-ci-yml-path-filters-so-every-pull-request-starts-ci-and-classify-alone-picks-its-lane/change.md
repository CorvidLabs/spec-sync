---
id: drop-the-ci-yml-path-filters-so-every-pull-request-starts-ci-and-classify-alone-picks-its-lane
state: implementing
type: operations
base_commit: 7ae7245f97e7cfd66cd80ce39b611becb1095642
---

# Drop the ci.yml path filters so every pull request starts CI and classify alone picks its lane

## Intent

Drop the ci.yml path filters so every pull request starts CI and classify alone picks its lane

## Affected Canonical Specs

- `github`

## Acceptance Criteria

- The `github` spec gains REQ-github-026: the CI workflow starts on every pull request to main and every push to main with no path filter, and `classify-ci-paths.sh` alone picks the lane, so a pull request that changes only paths it routes, `.chough/` records and `scripts/` included, still gets the required `Required CI gate` (CI-6); the spec version, the testing table row that relied on the path filters, and the Change Log follow.
- `.github/workflows/ci.yml` has no `paths:` or `paths-ignore:` filter on its `pull_request` or `push` trigger; both keep `branches: [main]`, and `workflow_dispatch` stays.
- `.github/scripts/test-ci-trigger-paths.py` fails when either trigger has a path filter, naming the recovery, and proves with the real classifier and the required-gate model from `test-required-ci-gate.py` that a change touching only Chough records, or only `scripts/`, is classified into a lane whose simulated run reports `Required CI gate` green on both events; it also fails when the fledge `verify` and `ci` lanes or `ci.yml` stop running it.
- `hi/ci.md` gains CI-6 in Leif's words, "A pull request that only changes Chough records still gets the checks the branch requires.", with its source in the Intent section, and the `INTENT.md` index is regenerated with `hi index`.
- Failing test first: on the unmodified workflow the new test fails (both triggers carry `paths:`, so neither change starts CI, and the test is not wired into fledge or CI); after the fix it passes, and `test-required-ci-gate.py` and `test-classify-ci-paths.sh` still pass. No tag or release is created or published.

## No-spec Rationale

Not applicable
