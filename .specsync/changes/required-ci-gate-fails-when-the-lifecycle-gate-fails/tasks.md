---
change: required-ci-gate-fails-when-the-lifecycle-gate-fails
artifact: tasks
---

# Tasks

- [x] Add `preflight` and `lifecycle-gate` to `implementation-gate.needs`.
- [x] Accept `skipped` only where the job's own `if:`, evaluated over classify outputs, deselected it; fail a selected job that was skipped.
- [x] Keep the old failure/cancelled check and fail when rows and `needs` disagree in number.
- [x] Guard test: missing `preflight`/`lifecycle-gate`, a job missing from `needs`, a row that drifts from its job's `if:`.
- [x] Simulate every classify lane, flag combination and event against the gate's own script.
- [x] Reproduce #796 against the pre-fix gate, and show the test fails on the pre-fix `ci.yml`.
- [x] Wire the test into `validate-action` and the Fledge `verify`, `ci` and `repo` lanes.
- [x] `docs/HLD.md`, the `github` delta, and the `github` companion notes.
- [x] `fledge lanes run verify`, `fledge lanes run pre-push` and `fledge trust verify`.
- [ ] Confirm on the pull request that `Required CI gate` is red while the draft is unapproved.
