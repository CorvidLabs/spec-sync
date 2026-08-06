---
change: CHG-0087-decide-pull-requests-in-seconds-instead-of-minutes
artifact: tasks
---

# Tasks

- [x] Add a git-only `preflight` job that rejects orphaned verification evidence
- [x] Add a `lifecycle-gate` job running `change audit --strict`
- [x] Make the expensive jobs depend on the gate
- [x] Add the same audit ahead of the trust contract gate
- [x] Override the step shell so the script owns its exit status
