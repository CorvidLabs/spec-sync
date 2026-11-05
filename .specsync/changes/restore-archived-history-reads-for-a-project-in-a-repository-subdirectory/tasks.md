---
change: restore-archived-history-reads-for-a-project-in-a-repository-subdirectory
artifact: tasks
---

# Tasks

- [x] Add the subdirectory fixture and confirm both new tests fail on main.
- [x] Anchor the dated-archive glob with `:(top,glob)` and the history walk of those paths with `:(top,literal)`.
- [x] Anchor `lifecycle_paths_touched_since` with `:(top,literal)`.
- [x] Record the restored behavior on REQ-change-043 and REQ-change-119, and in the change spec.
- [ ] Leif approves the definition, then reviews and ships. This session does not record those gates.
