---
change: drop-the-ci-yml-path-filters-so-every-pull-request-starts-ci-and-classify-alone-picks-its-lane
artifact: testing
---

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
