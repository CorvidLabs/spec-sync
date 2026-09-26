## MODIFIED

### SPEC SECTION Invariants

Every path that can be merged can reach the required CI gate; a path the CI
workflow cannot trigger can never report the gate and blocks its pull request.

The required CI gate fails whenever a job it depends on did not succeed, unless classify
deselected that job. A skipped job passes only when its own `if:` condition, evaluated over the
classify outputs, left it unselected. A selected job that was skipped was skipped because a job it
needs did not succeed, and it fails the gate: `skipped` is never read as green on its own. The
lifecycle preflight and the lifecycle gate are selected on every path, so either one failing turns
the required gate red.

Release qualification verifies exactly the tag protections this repository actually has, and names
every protection it does not verify on every run, green runs included. A gate that demands an
unprovisioned policy fails on every candidate and therefore verifies nothing — it is not a safe
default, because the protections that DO exist are never reached. Dropping a check from the gate is
permitted; dropping it silently is not. The tag protections that remain admit no bypass actor and
no broadening — where that can be observed. GitHub returns `bypass_actors` only to a caller with
admin access to repository settings, and the workflow token is not one, so the field is ABSENT
from every payload CI fetches. Absence means UNOBSERVED, never "no bypass actors": it is checked
when visible, refused when it grants anyone, and named in the unenforced disclosure when it cannot
be read. Requiring it made the gate impossible to satisfy from CI, which is how a lane stayed red
on every candidate while appearing to enforce something.

Release authority is stated wherever it is exercised. The final tag is created by the release
workflow's own token under a permission scoped to the single job that writes it, so the authority
to run the release lane is the authority to create a release tag; that equivalence is announced by
every run and recorded at the job itself, never left to be inferred from a green result. A named
deployment environment that does not exist is not a gate — GitHub materializes it unprotected on
first use — so the workflow names no environment rather than publish a gate that gates nothing.

## ADDED

### REQUIREMENT REQ-github-021

The required CI gate SHALL fail whenever the lifecycle preflight, the lifecycle gate, or any job
classify selected for the run fails, is cancelled, or is skipped because a job it needs did not
succeed.

Acceptance Criteria

- `implementation-gate` (SpecSync implementation ready) needs `classify`, `preflight`,
  `lifecycle-gate` and every other job that can finish before it, and `ci-gate` (Required CI gate)
  passes only when `implementation-gate` succeeds.
- A skipped job passes only when that job's own `if:` condition, evaluated again over the classify
  outputs, deselected it.
- Full, site-only, VS Code-only, specs/lifecycle-only, archive-only, legacy archive-only and
  review-only pull requests, pushes to `main` and `workflow_dispatch` runs stay green when every
  selected job succeeds.
- `.github/scripts/test-required-ci-gate.py` fails when `preflight` or `lifecycle-gate` is missing
  from the gate's `needs`, when a job that gates on `lifecycle-gate` or can otherwise finish before
  the gate is missing, or when a gate row no longer matches its job's `if:`.
- The same test runs the gate's own script over every classify path and requires the required gate
  to be red when any one selected job fails or is cancelled.
