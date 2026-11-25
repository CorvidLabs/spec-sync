---
change: release-candidate-qualification-runs-as-parallel-parts-within-30-minutes-and-no-workflow-job-has-a-longer-limit
artifact: tasks
---

# Tasks

- [x] Failing contract tests first: the validator's `parts` mode, the workflow time limits, the rehearsal lockstep, the `qualify` matrix.
- [x] The part lanes, the part script and the validator's `parts` mode.
- [x] `release.yml` runs ten parts within 30 minutes each and builds the platform evidence from their records.
- [x] The rehearsal workflow, and every workflow job at 30 minutes or less.
- [x] The runbook, the CI confidence notes, the agent instructions and the changelog.
- [x] The rehearsal on the pull request shows every part within its limit on hosted Ubuntu and macOS.
- [x] Targeted tests, the full native Verify lane, `change audit` and `check --strict` before review.
