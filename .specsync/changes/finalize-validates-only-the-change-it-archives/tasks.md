---
change: finalize-validates-only-the-change-it-archives
artifact: tasks
---

# Tasks

- [x] Measure the cold finalize Git-process growth on the #842 base.
- [x] Commit the failing finalize tests first.
- [x] Add the structural roster and switch the finalize/archive roster loads to it.
- [x] Validate a relied-on archive's own history before authenticating it.
- [x] Add `audit_project_with_strict` and wire `change audit --strict` to it, with its test.
- [x] Update specs, run the verify lane, `change check --commit`, and time finalize against the #842 build.
