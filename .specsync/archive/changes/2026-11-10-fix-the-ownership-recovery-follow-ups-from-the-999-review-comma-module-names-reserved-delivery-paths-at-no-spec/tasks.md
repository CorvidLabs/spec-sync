---
change: fix-the-ownership-recovery-follow-ups-from-the-999-review-comma-module-names-reserved-delivery-paths-at-no-spec
artifact: tasks
---

# Tasks

- [x] Exempt reserved delivery metadata from approve's production-ownership refusal (#1011.2)
- [x] Never print a comma-joined `affected_specs` answer when a module name cannot round-trip (#1011.1)
- [x] Fail the owner recovery closed when the spec listing exceeds its 10,000-entry bound (#1011.3)
- [x] Name the constraints re-answer in ownership recovery when it clears the answer (#1011.4)
- [x] Include registry-registered canonical specs outside specs_dir in owner discovery (#1011.5)
- [x] Skip the file-level ownership check for declared paths that are directories (#1011.6)
- [x] Tests for each; spec delta for REQ-change-053; spec + 3md updates
- [x] Review round (corvid-agent at c79a5727): recovery also refuses names `change answer` trims or `validate_module_name` rejects; unsafe `--spec` hints are quoted; test that `correct-owner --all-missing` skips reserved delivery paths; Change Log row matches version 223
