---
id: record-the-attestation-on-every-green-push-to-main-and-keep-the-pull-request-summary-red-when-a-gate-failed-or-was
state: archived
type: operations
base_commit: b3ca5f97aa9c07b09ab6bfd8d7eece28f2c30f79
---

# Record the attestation on every green push to main, and keep the pull request summary red when a gate failed or was blocked

## Intent

Record the attestation on every green push to main, and keep the pull request summary red when a gate failed or was blocked

## Affected Canonical Specs

- `github`

## Acceptance Criteria

- The `github` spec gains REQ-github-029. `attest` ("Record attestation") runs on every push to main whose `Required CI gate` succeeded, and on no other event or ref. Its condition starts with `always()`, so GitHub's implicit job-level `success()` over every upstream job (`corvid-pet` is skipped on every push) cannot skip it (CI-9).
- The `github` spec gains REQ-github-030. `corvid-pet`'s summary has exactly one row for every job it waits on except `classify`, `preflight` and `lifecycle-gate` included. Each row carries that job's own selection, held to the job's `if:` by the same rule as `implementation-gate`'s GATES rows. A skipped job reads "Not selected" only when classify deselected it. A selected job that was skipped reads as blocked and turns the summary red, as any failure does (CI-10).
- `.github/scripts/test-ci-attest-and-summary.py` reuses `test-required-ci-gate.py`'s workflow model and simulation. It fails when `attest` is skipped on a green push to main, runs off a push to main, or runs after the required gate failed. It also fails when the summary lacks a row for a job it waits on, a row's selection differs from its job's `if:`, a failed lifecycle gate or preflight leaves the summary green or calls a blocked job "Not selected", or the test stops running in the fledge `verify` and `ci` lanes or `ci.yml`.
- `hi/ci.md` gains CI-9, "Every commit on main that passes its required checks gets its attestation recorded.", and CI-10, "The summary on a pull request never looks green when a check that gates it failed or was blocked.", in Leif's words with their source in the Intent section, and `INTENT.md` is regenerated with `hi index`.
- Failing test first: on the unmodified workflow the new test fails 10 subtests (attest skipped on all 4 green push-to-main scenarios; a failed lifecycle gate or preflight gives a green summary; rows carry no selection; not wired in). After the change it passes, and `test-required-ci-gate.py`, `test-ci-trigger-paths.py` and `test-linux-baseline.py` still pass. `Required CI gate` and `implementation-gate` are unchanged. No tag or release is created or published.

## No-spec Rationale

Not applicable
