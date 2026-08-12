---
change: CHG-0106-make-verification-currency-a-content-question-delete-the-git-ancestry-walk-the
artifact: tasks
---

# Tasks

- [x] Split the currency check at the content/history line
- [x] Reduce `validate_verification_for_commit_binding` to content; update both call sites
- [x] Delete the ancestry walk, path allowlist, and consistency check whole
- [x] Delete the 8 tests describing removed behaviour rather than adapting them
- [x] Update REQ-change-013 and REQ-change-016
- [x] CHANGELOG: squash fix, deadlock fix, reduced change-then-revert detection
- [x] Sandbox drills against this build: 038 10/10, 028 15/15, 036 8/8, 032 7/7
