# Lesson bundle: record-the-attestation-on-every-green-push-to-main-and-keep-the-pull-request-summary-red-when-a-gate-failed-or-was

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Record the attestation on every green push to main, and keep the pull request summary red when a gate failed or was blocked
- **Kind**: Operations
- **Specs**: github
- **Paths**: .github/workflows/ci.yml, .github/scripts/test-ci-attest-and-summary.py, fledge.toml, hi/ci.md, INTENT.md, specs/github
- **Acceptance**: The `github` spec gains REQ-github-029. `attest` ("Record attestation") runs on every push to main whose `Required CI gate` succeeded, and on no other event or ref. Its condition starts with `always()`, so GitHub's implicit job-level `success()` over every upstream job (`corvid-pet` is skipped on every push) cannot skip it (CI-9).
- **Acceptance**: The `github` spec gains REQ-github-030. `corvid-pet`'s summary has exactly one row for every job it waits on except `classify`, `preflight` and `lifecycle-gate` included. Each row carries that job's own selection, held to the job's `if:` by the same rule as `implementation-gate`'s GATES rows. A skipped job reads "Not selected" only when classify deselected it. A selected job that was skipped reads as blocked and turns the summary red, as any failure does (CI-10).
- **Acceptance**: `.github/scripts/test-ci-attest-and-summary.py` reuses `test-required-ci-gate.py`'s workflow model and simulation. It fails when `attest` is skipped on a green push to main, runs off a push to main, or runs after the required gate failed. It also fails when the summary lacks a row for a job it waits on, a row's selection differs from its job's `if:`, a failed lifecycle gate or preflight leaves the summary green or calls a blocked job "Not selected", or the test stops running in the fledge `verify` and `ci` lanes or `ci.yml`.
- **Acceptance**: `hi/ci.md` gains CI-9, "Every commit on main that passes its required checks gets its attestation recorded.", and CI-10, "The summary on a pull request never looks green when a check that gates it failed or was blocked.", in Leif's words with their source in the Intent section, and `INTENT.md` is regenerated with `hi index`.
- **Acceptance**: Failing test first: on the unmodified workflow the new test fails 10 subtests (attest skipped on all 4 green push-to-main scenarios; a failed lifecycle gate or preflight gives a green summary; rows carry no selection; not wired in). After the change it passes, and `test-required-ci-gate.py`, `test-ci-trigger-paths.py` and `test-linux-baseline.py` still pass. `Required CI gate` and `implementation-gate` are unchanged. No tag or release is created or published.

## Evidence

- Verification commit: `c6cd589e74875833c38559d7d2cb6a350f60d2ea`
- Base commit: `b3ca5f97aa9c07b09ab6bfd8d7eece28f2c30f79`
- Verified by: `specsync check --spec github`

## From the change's context.md

# Context

Two CI defects from the #796/#797 work, grouped as G-ci-jobs in the 2026-11-06 triage.

- **#798:** `Record attestation` (`attest`) was skipped on every push to main, including green ones.
  - Its `if:` (`needs.ci-gate.result == 'success' && github.event_name == 'push' && github.ref == 'refs/heads/main'`) has no status function, so GitHub ANDs in an implicit `success()`.
  - At job level, `success()` looks at every transitive dependency. `ci-gate` needs `implementation-gate`, which needs `corvid-pet`, and `corvid-pet` runs only on pull requests.
  - So on a push `success()` is false and `attest` is skipped.
  - `test-required-ci-gate.py`'s model already simulates this and says so in a comment.
- **#799:** the scoped review's summary (`corvid-pet`, step `status`) builds its table from `CHECK_*` rows.
  - It has no row for `preflight` or `lifecycle-gate`, and it labels every `skipped` result "Not selected".
  - A failed lifecycle gate skips `test`, `audit`, `coverage` and `spec-check`, and the summary still reads green. After #797 the merge is blocked, but the review comment is misleading.

HI: CI-9 and CI-10 are new, in Leif's words ("Both A", Herdr orc tab, 2026-11-06, relayed by orc t18). REQ-github-029 and REQ-github-030 were claimed with the orc.

Ruled out:
- Dropping `corvid-pet` from `implementation-gate`'s needs. #796 requires every product job that can finish before the gate to be in its needs.
- Moving `attest` off `ci-gate`. Attestation must follow the required gate.

## From the change's design.md

# Design

**attest.** `if: ${{ always() && needs.ci-gate.result == 'success' && github.event_name == 'push' && github.ref == 'refs/heads/main' }}`.
- `always()` stops the implicit transitive `success()`.
- The explicit `needs.ci-gate.result == 'success'` keeps the attestation behind the required gate, which itself needs every gate job.
- `needs` stays `[ci-gate]`, and the step is unchanged and best-effort.

**Summary rows.** Each `CHECK_*` value becomes `<label>=<selected>=<result>`. `<selected>` is `true`, or the job's own `if:` less a leading `always() &&`, as a `${{ }}` expression. It is the same selection `implementation-gate`'s GATES rows carry.
- Two rows are added: `CHECK_PREFLIGHT` ("Lifecycle preflight") and `CHECK_LIFECYCLE` ("Lifecycle gate"). Both are always selected.
- The script splits each value into label, selection and result:
  - `success` → ✅ Passed
  - `false:skipped` → ⏭️ Not selected
  - `true:skipped` → ⛔ Blocked (a job it needs did not succeed), and the summary fails
  - anything else → ❌ \<result\>, and the summary fails
- Labels contain no `=`.
- `REPORT_SPEC`, the details block and the corvid-pet Action inputs are unchanged.

**Test.** `.github/scripts/test-ci-attest-and-summary.py` imports `test-required-ci-gate.py` as a module, as `test-ci-trigger-paths.py` does.
- **attest:** `GATE.simulate` over the model's own `SCENARIOS`.
- **Summary:** it parses the `CHECK_*` rows, holds each selection to `GATE.selection_expression` of its job, and requires one row per waited-on job.
- **Running the real step:** it runs the step's real script under `bash -e` with `GITHUB_OUTPUT` set, over env values interpolated from a simulated run. That covers a green full pull request, a failed lifecycle gate, a failed preflight, and hand-built deselected and blocked rows.
- It runs as fledge task `ci-attest-and-summary-test` in the `ci`, `verify` and `repo` lanes, and as a `validate-action` step in `ci.yml`.

## From the change's testing.md

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

## Where these lessons go

- `specs/github/github.3md`
