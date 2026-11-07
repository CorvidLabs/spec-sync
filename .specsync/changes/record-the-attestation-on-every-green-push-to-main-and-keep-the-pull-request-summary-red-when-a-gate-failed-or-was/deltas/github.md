## ADDED

### REQUIREMENT REQ-github-029

`Record attestation` (`attest`) SHALL run on every push to main whose `Required CI gate` succeeded, and on no other event or ref.

- Its `if:` SHALL start with `always()` and require `needs.ci-gate.result == 'success'`, a `push` event and `refs/heads/main`. Without `always()`, GitHub's implicit job-level `success()` covers every job upstream of `ci-gate`. `corvid-pet` is one of them and is skipped on every push, so `attest` was skipped on every push (CI-9, #798).
- It SHALL stay best-effort: a failure to download, sign or push logs a warning and never fails CI.

#### Acceptance Criteria

- Simulated with `test-required-ci-gate.py`'s workflow model, `attest` runs on every push-to-main scenario where `Required CI gate` succeeds.
- It is skipped when the required gate fails, and on every pull request and `workflow_dispatch` scenario.

### REQUIREMENT REQ-github-030

The SpecSync scoped review's pull-request summary (`corvid-pet`) SHALL NOT read green when a check that gates the pull request failed or was blocked.

- The summary SHALL have exactly one row for every job the review waits on, except `classify`. That includes `preflight` and `lifecycle-gate`.
- Each row SHALL carry the job's own selection, held to the job's `if:` by the same rule as `implementation-gate`'s GATES rows.
- A skipped job SHALL read "Not selected" only when classify deselected it. A selected job that was skipped SHALL read as blocked and turn the summary red, as any failure does (CI-10, #799).

#### Acceptance Criteria

- A test fails when the summary lacks a row for a job it waits on, or when a row's selection differs from its job's `if:`.
- Simulating a failed `lifecycle-gate`, or a failed `preflight`, on a full pull request yields a red summary that calls no blocked job "Not selected". A green run yields a green summary.
- The fledge `verify` and `ci` lanes and `ci.yml` run that test, and the test fails when they stop.
