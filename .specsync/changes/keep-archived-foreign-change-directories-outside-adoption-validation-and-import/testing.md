---
change: keep-archived-foreign-change-directories-outside-adoption-validation-and-import
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-014 | `tests/integration/change.rs::adopt_dry_run_and_import_skip_archive_descendants_but_refuse_active_symlinks` runs SpecSync in disposable OpenSpec and Spec Kit projects, validates both modes and checks the destination contents. |
