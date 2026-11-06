---
change: drop-the-ci-yml-path-filters-so-every-pull-request-starts-ci-and-classify-alone-picks-its-lane
artifact: plan
---

# Plan

## Approach

Remove the `paths:` lists from the `pull_request` and `push` triggers in `ci.yml`, and leave the job selection to `classify-ci-paths.sh`. A new lockstep test holds both rules, and it runs in fledge and in CI.

## Out of scope

- The classifier's lanes. No path changes lane.
- The other workflows' triggers: `trust.yml`, `pages.yml`, `release.yml`, `rc-assets.yml` and the lifecycle workflows.
- Chough itself: the session pile-up and native-read timeouts seen in #941 are reported there.
- #941's own merge. It merges `main` in after this lands, and then CI starts on it.

## Steps

1. Capture CI-6 in `hi/ci.md` with its source, and run `hi index` (done before approval).
2. Write `.github/scripts/test-ci-trigger-paths.py` and see it fail on the unmodified workflow (done before approval).
3. After approval:
   - remove both `paths:` lists from `ci.yml`;
   - add a `ci-trigger-paths-test` fledge task to the `ci`, `verify` and `repo` lanes;
   - run the test from `validate-action` in `ci.yml`.
4. Add REQ-github-026 to the `github` spec. Update the `specs/github/testing.md` row that relied on the path filters, plus the spec version and Change Log, through `change check`.
5. Run the new test, `test-required-ci-gate.py`, `test-classify-ci-paths.sh`, `change audit --strict`, the pre-push gate and the verify lane.

## Risks

- More CI runs: a pull request that touches only previously unlisted paths now runs the full lane. Those paths are rare (agent skill installs, `scripts/`, root config), and the full lane is what classify already chose for them.
- Every push to main now starts CI, including pushes that touch only those paths.

## Constraints consulted

- `Required CI gate` is the only required check on main, and `test-required-ci-gate.py` holds its contract over every classify lane (#796). The new test reuses that model rather than copying it.
- CI is archive-first.
- Workflow tests read YAML through Ruby's standard Psych.
