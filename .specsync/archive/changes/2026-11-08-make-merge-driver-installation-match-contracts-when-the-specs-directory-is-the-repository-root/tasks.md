---
change: make-merge-driver-installation-match-contracts-when-the-specs-directory-is-the-repository-root
artifact: tasks
---

# Tasks

- [x] Add the root-specs `git check-attr` regression test and record its failure on the unmodified base.
- [x] Normalize `.` and `./` to repository-root attribute patterns without changing non-root installation behavior.
- [x] Update the `cmd_merge` requirement, spec invariant and change log, module tasks, and `cmd_merge.3md` lesson.
- [x] Pass focused install tests and the original `merge --install` plus `git check-attr` scenario.
- [x] Pass `fledge lanes run verify` and pre-push.

Human scope approval, implementation review, and archive-tip CI are delivery gates, not task checkboxes.
