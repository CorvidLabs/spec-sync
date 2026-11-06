---
change: close-a-change-that-merged-but-was-never-archived-in-one-audited-late-close-that-records-it-as-never-re-checked
artifact: tasks
---

# Tasks

- [ ] Write the failing tests for REQ-change-143 and record that they fail on main f7180c34.
- [ ] Add the `closed_late` state and decide every exhaustive match.
- [ ] Add `change close-late` with the merge proof, `late-close.json`, the all-or-nothing write and `--dry-run`.
- [ ] Report the late close in status, ship-status, JSON and audit.
- [ ] Update the contracts, docs, AGENTS.md quick reference and CHANGELOG.
- [ ] Materialize with change check --commit, pass audit --strict, the verify lane and pre-push.

Human gates (not tasks): scope approval and implementation review, recorded by Leif through the orc.
