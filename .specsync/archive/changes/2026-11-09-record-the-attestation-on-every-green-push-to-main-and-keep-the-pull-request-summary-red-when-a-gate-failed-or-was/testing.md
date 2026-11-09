---
change: record-the-attestation-on-every-green-push-to-main-and-keep-the-pull-request-summary-red-when-a-gate-failed-or-was
artifact: testing
---

# Testing

| Case | Expected | How |
|---|---|---|
| Green push to main (full, verifying change, site-only, specs-only) | `attest` runs | `test_attest_runs_on_every_main_push_whose_required_gate_passed`: `GATE.simulate` over the model's push scenarios |
| Required gate fails on a push | `attest` skipped | `test_attest_does_not_run_when_the_required_gate_fails` (`test` forced to fail) |
| Pull requests and dispatch | `attest` skipped | `test_attest_never_runs_off_a_push_to_main` |
| A job the summary waits on has no row | The test fails | `test_the_summary_has_one_row_for_every_job_it_waits_on` |
| A row's selection differs from its job's `if:` | The test fails | `test_each_row_selects_on_its_jobs_own_condition` (`GATE.selection_expression`) |
| Lifecycle gate or preflight fails on a full PR | Summary red, no blocked job reads "Not selected" | `test_a_failed_lifecycle_gate_or_preflight_turns_the_summary_red`: simulated results, then the real step script under `bash -e` |
| Everything green | Summary green | `test_a_green_run_has_a_green_summary` |
| Deselected vs blocked rows | "Not selected" passes; blocked is red | `test_deselected_reads_not_selected_and_blocked_does_not` |
| The test stops running | The test fails | `test_this_test_runs_through_fledge_and_ci` |

Failing first: on the unmodified workflow, 10 subtests fail across 6 tests.
- attest is skipped on all 4 green push-to-main scenarios.
- Both failure cases give a green summary.
- The deselected-vs-blocked case reads green.
- Rows carry no selection (2 tests).
- The test is not wired in.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-github-029 | `.github/scripts/test-ci-attest-and-summary.py`: `test_attest_runs_on_every_main_push_whose_required_gate_passed`, `test_attest_does_not_run_when_the_required_gate_fails`, `test_attest_never_runs_off_a_push_to_main` |
| REQ-github-030 | `.github/scripts/test-ci-attest-and-summary.py`: `test_the_summary_has_one_row_for_every_job_it_waits_on`, `test_each_row_selects_on_its_jobs_own_condition`, `test_a_failed_lifecycle_gate_or_preflight_turns_the_summary_red`, `test_a_green_run_has_a_green_summary`, `test_deselected_reads_not_selected_and_blocked_does_not`, `test_this_test_runs_through_fledge_and_ci` |
