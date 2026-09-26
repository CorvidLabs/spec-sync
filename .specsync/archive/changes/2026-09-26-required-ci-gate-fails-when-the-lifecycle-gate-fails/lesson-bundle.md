# Lesson bundle — required-ci-gate-fails-when-the-lifecycle-gate-fails

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Required CI gate fails when the lifecycle gate fails
- **Kind**: BugFix
- **Specs**: github
- **Paths**: .github/workflows/ci.yml, .github/scripts/test-required-ci-gate.py, fledge.toml, docs/HLD.md
- **Acceptance**: Required CI gate fails whenever Lifecycle preflight or Lifecycle gate fails or is cancelled. implementation-gate (SpecSync implementation ready) needs classify, preflight, lifecycle-gate and every other job that can finish before it, and accepts skipped for a job only when that job's own if: condition, evaluated again over the classify outputs, deselected it; a selected job that was skipped because something it needs did not succeed fails the gate. Full, site-only, VS Code-only, specs/lifecycle-only, archive-only, legacy archive-only and review-only pull requests, pushes to main and workflow_dispatch runs stay green when every selected job succeeds. .github/scripts/test-required-ci-gate.py runs in the validate-action CI job and in the Fledge verify lane; it fails if preflight or lifecycle-gate is missing from implementation-gate.needs, if a job that gates on lifecycle-gate or can otherwise finish before the gate is missing, or if a gate row no longer matches its job's if:, and it simulates every classify path to show the required gate red when any one selected job fails or is cancelled and green otherwise, and reproduces #796 against the pre-fix gate.

## Evidence

- Verification commit: `3ffef4758ab6654507183cab5942b412dc6064fa`
- Base commit: `cddc39e478dcc1f111940a3cfb02134bba9804cc`
- Verified by: `specsync check --spec github`

## From the change's context.md

# Context

`Required CI gate` is the only required check on `main`, and it passes when
`implementation-gate` ("SpecSync implementation ready") passes. On #795 at `bb1d80f2`,
`Lifecycle gate` and `trust` failed, `test`, `audit`, `coverage` and `spec-check` never ran,
and both gates were green (#796). orc found it while shipping #795.

The cause is in `.github/workflows/ci.yml`:

- `test`, `audit`, `coverage` and `spec-check` need `lifecycle-gate`. When it fails, GitHub
  reports them as `skipped`, not failed.
- `implementation-gate` needed neither `preflight` nor `lifecycle-gate`, and it accepted
  `success` or `skipped` from every job it did need.
- `ci-gate` only required `implementation-gate == success`.

So a failed lifecycle gate became four skipped jobs, and skipped read as green.

What a session picking this up needs to know:

- `skipped` alone cannot say why a job did not run. GitHub reports the same result for a job
  classify deselected and for a job whose dependency failed. The gate can only tell them apart by
  asking the question the job asked, so each gate row carries the job's own `if:` condition,
  evaluated again over the same classify outputs.
- Classify outputs are fixed once `classify` finishes, and the gate and the job use the same
  expression evaluator. The only way the two answers can differ is if the row and the job's `if:`
  are different text. The test compares them as text.
- `preflight` and `lifecycle-gate` have no `if:`, so they are selected on every path, archive-only
  and review-only included. Recent archive-only and review-only pull requests (#785, #790, #791)
  and the #795 product and archive tips all show `Lifecycle gate: success`, so requiring success
  blocks nothing that merges today.
- Job-level `success()` looks at every transitive dependency, not only the direct ones. That is
  why `attest` has been skipped on every push to `main` (it needs `ci-gate`, and `corvid-pet`,
  two levels up, is skipped on pushes). The simulation models this. Fixing `attest` is out of
  scope here and is reported separately.
- `act` is not installed and CI cannot be run locally, so the proof is a simulation of the job
  graph that executes the gate's own bash under GitHub's default invocation, plus the live run on
  this pull request.
- Ruled out: requiring `success` from every job in `needs`. Archive-only and review-only pull
  requests legitimately skip most jobs, and that would block them.

## From the change's design.md

# Design

## The gate asks each job's own question

`implementation-gate` needs every job that can finish before it: `classify`, `preflight`,
`lifecycle-gate`, the product jobs and `corvid-pet`. Its single step reads a `GATES` table with
one row per job:

```text
<job> <selected> <result>
test ${{ needs.classify.outputs.full == 'true' }} ${{ needs.test.result }}
```

`<selected>` is the job's own `if:` with any leading `always() &&` removed, or `true` for a job
with no `if:`. GitHub renders it as `true` or `false`. The step then applies:

| selected | result | verdict |
|---|---|---|
| true | success | pass |
| false | skipped | pass (classify deselected it) |
| true | skipped | **fail**: a dependency did not succeed |
| true | failure, cancelled | **fail** |
| false | anything but skipped | **fail**: the row no longer matches the job's `if:` |
| anything else | any | **fail**: malformed row |

It also keeps the previous check, so any `failure` or `cancelled` in `join(needs.*.result)` fails,
and it fails when the number of rows differs from the number of jobs in `needs`. `ci-gate` is
unchanged: it passes only when `implementation-gate` succeeds.

## Why the row repeats the condition rather than the rule living in a script

The gate evaluates the same expression, over the same fixed classify outputs, with the same
evaluator as the job's `if:`. It cannot disagree with the job unless the text differs, and the
test compares the text. A script outside the workflow would need a checkout in the gate and a
second copy of the lane rules.

## The guard

`.github/scripts/test-required-ci-gate.py` parses `ci.yml` with Psych, like the other workflow
validators, and checks:

1. Every job that does not itself depend on `implementation-gate` is in its `needs`. Any new job
   that gates on `lifecycle-gate`, or is otherwise selected before the gate, fails the test until
   it is added. `classify`, `preflight` and `lifecycle-gate` are named explicitly.
2. Every job in `needs` has exactly one row, each row reads its own job's result, and its
   selection text equals the job's `if:`. A job whose `if:` uses a status function other than a
   leading `always()` fails, because the gate cannot mirror it.
3. A simulation of the job graph, with GitHub's implicit and transitive `success()`, runs the
   gate's own step and `ci-gate`'s step under `bash -e`, as the runner does for a step that names
   no shell. It covers every classify lane, every combination of the classify flags the
   conditions read, and every event. The required gate must be green when every selected job
   succeeds, and red when any one of them fails or is cancelled.
4. The pre-#796 gate definition, kept as a fixture, reproduces the bug in the same simulation, so
   the harness can see what it guards against.

It runs in the `validate-action` CI job, which runs whenever `ci.yml` changes because a workflow
change selects the full lane, and as the Fledge task `ci-gate-test` in the `verify`, `ci` and
`repo` lanes.

## From the change's testing.md

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
  the steps of `implementation-gate` and `ci-gate` run under `bash -e`, which is how the runner
  invokes a step that names no shell (the job log prints `shell: /usr/bin/bash -e {0}`). On 15
  named lanes (full, full awaiting review, site-only,
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

The same tests run against `ci.yml` from `origin/main` (`cddc39e4`) fail 65 cases in five tests:
the contract (`preflight` and `lifecycle-gate` missing, no `GATES` table), both named `needs`
checks, the CI wiring check, the #796 reproduction, and 60 injection cases (`preflight` or
`lifecycle-gate` failing or cancelled, on each of the 15 lanes). On this branch all pass.

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

The pull request (#797) was opened before approval, so `Lifecycle gate` failed on the unapproved
draft at `17e63f5d` (run 36252382792): "meaningful changed paths are not covered by an active
change". `test`, `audit`, `coverage` and `spec-check` were skipped, as on #795. This time
`SpecSync implementation ready` failed, with one annotation per cause:

```text
lifecycle-gate was selected and ended with: failure
test was selected but skipped, so a job it needs did not succeed
spec-check was selected but skipped, so a job it needs did not succeed
audit was selected but skipped, so a job it needs did not succeed
coverage was selected but skipped, so a job it needs did not succeed
a job this gate needs ended with: failure
```

`Required CI gate` failed with it. The new test ran in `validate-action` (29 tests, OK). The same
job log shows the step shell as `bash -e {0}`; the simulation first assumed
`bash --noprofile --norc -eo pipefail`, which is what `shell: bash` gets, and now uses `bash -e`.

## Where these lessons go

- `specs/github/context.md`
