---
change: make-merge-driver-installation-match-contracts-when-the-specs-directory-is-the-repository-root
artifact: tasks
---

# Tasks

- [ ] Add the root-specs `git check-attr` regression test and record its failure on the unmodified base.
- [ ] Normalize `.` and `./` to repository-root attribute patterns without changing non-root installation behavior.
- [ ] Update the `cmd_merge` requirement, spec invariant and change log, module tasks, and `cmd_merge.3md` lesson.
- [ ] Pass focused install tests and the original `merge --install` plus `git check-attr` scenario.
- [ ] Pass `fledge lanes run verify`, pre-push, and the archive-tip CI lane.

Human scope approval and implementation review are orc-recorded gates, not task checkboxes.
