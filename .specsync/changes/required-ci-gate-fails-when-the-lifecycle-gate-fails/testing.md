---
change: required-ci-gate-fails-when-the-lifecycle-gate-fails
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-github-021 | `.github/scripts/test-required-ci-gate.py`: `test_workflow_holds_to_the_gate_contract`, `test_gate_needs_preflight_and_the_lifecycle_gate`, `test_every_job_gated_on_the_lifecycle_gate_is_required`, `test_issue_796_lifecycle_gate_failure_turns_the_required_gate_red`, `test_any_selected_job_failing_or_cancelled_turns_the_required_gate_red`, `test_every_path_is_green_when_every_selected_job_succeeds`, `test_every_classify_output_combination_is_green_when_everything_succeeds`, `test_deselected_jobs_are_skipped_and_do_not_block`, the seven `GuardMutationTests` and the six `GateScriptTests` |

## What the tests do

- **Contract.** `ci.yml` is parsed with Psych. Every job that does not depend on
  `implementation-gate` must be in its `needs`, and `classify`, `preflight` and `lifecycle-gate`
  are named. Every job in `needs` has one `GATES` row that reads its own result and whose
  selection text equals the job's `if:`.
- **Guard mutations.** Each of these fails the contract: dropping `lifecycle-gate` or
  `preflight` from `needs`, adding a job with `needs: [classify, lifecycle-gate]` or a new
  site-selected job without adding it, adding a job to `needs` without a row, changing `test`'s
  `if:` without its row, and an `if:` that calls `failure()`.
- **Simulation.** The job graph is evaluated with GitHub's implicit, transitive `success()`, and
  the steps of `implementation-gate` and `ci-gate` run under
  `bash --noprofile --norc -eo pipefail`. On 15 named lanes (full, full awaiting review, site-only,
  VS Code-only, site and VS Code, specs/lifecycle-only with and without a review due, archive-only,
  legacy archive-only, review-only, four push-to-`main` lanes and `workflow_dispatch`), the
  required gate is green when every selected job succeeds. Forcing any one selected job to
  `failure` or `cancelled` turns it red in all 252 cases. All 192 combinations of the six classify
  flags the conditions read, under all three events, are green when everything succeeds.
- **Reproduction.** With the pre-#796 gate definition swapped in, a failed lifecycle gate on a
  full pull request leaves `test`, `audit`, `coverage` and `spec-check` skipped, and both gates
  green, which is what #795 showed at `bb1d80f2`.
- **Gate script.** Run directly, the step fails on a selected job that was skipped, a deselected
  job that ran, a malformed selection, rows that do not cover `needs`, and any `failure` in `needs`.

## Discrimination

The same tests run against `ci.yml` from `origin/main` (`cddc39e4`) fail 64 cases in four tests:
the contract (`preflight` and `lifecycle-gate` missing, no `GATES` table), both named `needs`
checks, the #796 reproduction, and 60 injection cases (`preflight` or `lifecycle-gate` failing or
cancelled, on each of the 15 lanes). On this branch all pass.

## Truth table

Printed by `python3 .github/scripts/test-required-ci-gate.py --truth-table` and recorded in the
pull request.

## Suite

`fledge lanes run verify` passes: fmt, `cargo clippy -- -D warnings` and `cargo check`; the full
`cargo test` with 2504 unit and 437 integration tests and 0 failures; the release build;
`specsync check --strict --require-coverage 100 --force` with 62/62 specs and 100% file coverage;
the 52 release-candidate tests; and the 29 new `ci-gate-test` tests.
`python3 -S .github/scripts/validate-workflow-runtime-pins.py`,
`python3 -S .github/scripts/validate-release-version.py` and
`.github/scripts/test-classify-ci-paths.sh` pass on the changed workflow.
`fledge lanes run pre-push` and `fledge trust verify` pass.

## Live check

The pull request is opened before approval, so `Lifecycle gate` fails on the unapproved draft.
With this change `SpecSync implementation ready` and `Required CI gate` must be red on that head.
