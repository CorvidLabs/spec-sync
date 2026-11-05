---
change: finalize-validates-only-the-change-it-archives
artifact: tasks
---

# Tasks

- [x] Measure the cold finalize Git-process growth on the #842 base.
- [ ] Commit the failing finalize tests first.
- [ ] Add the structural roster and switch the finalize/archive roster loads to it.
- [ ] Validate a relied-on archive's own history before authenticating it.
- [ ] Add `audit_project_with_strict` and wire `change audit --strict` to it, with its test.
- [ ] Update specs, run the verify lane, `change check --commit`, and time finalize against the #842 build.
