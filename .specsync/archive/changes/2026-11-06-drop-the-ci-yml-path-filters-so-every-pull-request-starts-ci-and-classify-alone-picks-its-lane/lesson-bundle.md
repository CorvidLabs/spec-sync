# Lesson bundle: drop-the-ci-yml-path-filters-so-every-pull-request-starts-ci-and-classify-alone-picks-its-lane

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Drop the ci.yml path filters so every pull request starts CI and classify alone picks its lane
- **Kind**: Operations
- **Specs**: github
- **Paths**: .github/workflows/ci.yml, .github/scripts/test-ci-trigger-paths.py, fledge.toml, specs/github/, hi/ci.md, INTENT.md, .github/scripts/test-linux-baseline.py
- **Acceptance**: The `github` spec gains REQ-github-026: the CI workflow starts on every pull request to main and every push to main with no path filter, and `classify-ci-paths.sh` alone picks the lane, so a pull request that changes only paths it routes, `.chough/` records and `scripts/` included, still gets the required `Required CI gate` (CI-6); the spec version, the testing table row that relied on the path filters, and the Change Log follow.
- **Acceptance**: `.github/workflows/ci.yml` has no `paths:` or `paths-ignore:` filter on its `pull_request` or `push` trigger; both keep `branches: [main]`, and `workflow_dispatch` stays.
- **Acceptance**: `.github/scripts/test-ci-trigger-paths.py` fails when either trigger has a path filter, naming the recovery, and proves with the real classifier and the required-gate model from `test-required-ci-gate.py` that a change touching only Chough records, or only `scripts/`, is classified into a lane whose simulated run reports `Required CI gate` green on both events; it also fails when the fledge `verify` and `ci` lanes or `ci.yml` stop running it.
- **Acceptance**: `hi/ci.md` gains CI-6 in Leif's words, "A pull request that only changes Chough records still gets the checks the branch requires.", with its source in the Intent section, and the `INTENT.md` index is regenerated with `hi index`.
- **Acceptance**: Failing test first: on the unmodified workflow the new test fails (both triggers carry `paths:`, so neither change starts CI, and the test is not wired into fledge or CI); after the fix it passes, and `test-required-ci-gate.py` and `test-classify-ci-paths.sh` still pass. No tag or release is created or published.

## Evidence

- Verification commit: `d1b07f53f1f55f53737d8e33369c45548da6c219`
- Base commit: `7ae7245f97e7cfd66cd80ce39b611becb1095642`
- Verified by: `specsync check --spec github`

## From the change's context.md

# Context

PR #941 changed only `.chough/` records, to close the Chough sessions of merged changes. It never started CI. `ci.yml` starts only when a changed path matches a glob in its `pull_request` or `push` `paths:` list, and `.chough/**` was not in that list. So the required `Required CI gate` never reported, and the PR stayed BLOCKED.

A `workflow_dispatch` run on the branch went down the full lane and was green, but GitHub leaves dispatch suites out of a pull request's status rollup, so it did not count.

The same gap covers every tracked path the 27 globs miss:
- `scripts/`;
- the agent skill installs: `.agents/`, `.claude/`, `.codex/`, `.cursor/`, `.gemini/`;
- `.github/CODEOWNERS`, the issue templates and the PR template;
- `.gitignore`, `.gitattributes`, `LICENSE`, `.trust.toml`, `.attest.json`, `.augur.toml` and `.atlasignore`.

`classify-ci-paths.sh` already sends every path it does not name to the full lane, so the filters only ever decided whether CI started at all, never which lane it took.

Decisions, relayed by orc t18:
- Leif (2026-11-06, Herdr orc tab), "Capture it, fix CI": capture CI-6 in `hi/ci.md` as "A pull request that only changes Chough records still gets the checks the branch requires.", and fix CI in a separate small PR, failing test first.
- Leif, "Drop path filters" (option C of three): remove `paths:` from both triggers, so every PR starts CI and classify alone picks the lane.
  - Option A, rejected: add `.chough/**` and keep a list of known gaps.
  - Option B, rejected: add about 15 globs.
  - The test must show that `ci.yml` has no path filter, and that a `.chough`-only change and a `scripts`-only change each classify to a lane that reports the Required CI gate.
  - CI-6 as captured covers it.

Chough could not open a session for this change. A worktree off `main` refuses `chough session new` until #941 merges, and #941 is the PR this change unblocks. So evidence lives in SpecSync and the PR body.

## From the change's design.md

# Design

There are two decisions in CI today, and this change leaves one of them.

| Decision | Before | After |
|---|---|---|
| Does CI start? | Only if a changed path matches one of 27 `paths:` globs (identical for `pull_request` and `push`) | Always, for every pull request to main and every push to main |
| Which jobs run? | `classify-ci-paths.sh`: named paths pick the site, vscode, specs/lifecycle, archive or review lanes; everything else goes to the full lane | Unchanged |

Removing the filter can only add runs; it never removes one. A pull request the filter used to skip now starts, and classify gives it the full lane. That is the lane classify already gave such paths whenever they shared a PR with a listed path.

The test does not copy either side's rules. It reads `ci.yml` with Ruby's Psych, as the other workflow tests do. It asks the real `classify-ci-paths.sh` about the two change sets #941 exposed. It then simulates the resulting lane with `test-required-ci-gate.py`'s own model (`load_workflow`, `Scenario`, `simulate`), which runs the gate's real shell steps.

## From the change's testing.md

# Testing

| Case | Expected | How |
|---|---|---|
| A trigger has a path filter | The test fails and names the recovery | `test_ci_has_no_path_filter`: fails on the unmodified `ci.yml` for both `pull_request` and `push` |
| A change touches only Chough records | CI starts, classify picks a lane, and `Required CI gate` reports green when every selected job passes | `test_chough_only_and_scripts_only_changes_report_the_required_gate`: eight `.chough/` record kinds go through the real classifier, then `test-required-ci-gate.py`'s `simulate` on both events |
| A change touches only `scripts/` | Same as above | Same test, with `scripts/pre-push-gate.sh` and `scripts/spec-check-fast.sh` |
| The test stops running | The test fails | `test_this_test_runs_through_fledge_and_ci`: a fledge task in the `verify` and `ci` lanes, plus a `ci.yml` step |
| Existing gate and classifier contracts | Unchanged | `test-required-ci-gate.py` and `test-classify-ci-paths.sh` still pass |

Failing first: on the unmodified workflow the new test fails 7 subtests:
- both triggers carry `paths:` (2);
- neither change starts CI on either event (4);
- the test is not wired into fledge or CI (1).

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-github-026 | `.github/scripts/test-ci-trigger-paths.py`: `test_ci_has_no_path_filter`, `test_chough_only_and_scripts_only_changes_report_the_required_gate` (the real classifier, then `test-required-ci-gate.py`'s `simulate` on `pull_request` and `push`), `test_this_test_runs_through_fledge_and_ci`; `.github/scripts/test-linux-baseline.py`: `test_ci_runs_on_rc_assets_changes` |

## Where these lessons go

- `specs/github/github.3md`
