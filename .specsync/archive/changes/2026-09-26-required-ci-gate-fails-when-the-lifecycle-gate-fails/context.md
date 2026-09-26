---
change: required-ci-gate-fails-when-the-lifecycle-gate-fails
artifact: context
---

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
