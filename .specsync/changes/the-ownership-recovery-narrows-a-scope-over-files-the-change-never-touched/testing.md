---
change: the-ownership-recovery-narrows-a-scope-over-files-the-change-never-touched
artifact: testing
---

# Testing

`the_ownership_recovery_narrows_a_scope_over_files_the_change_never_touched` runs on a temp git
project. It fails when the split is disabled. The existing recovery tests, whose unowned
inputs are all modified, are unchanged.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-cmd-change-008 | unit `the_ownership_recovery_narrows_a_scope_over_files_the_change_never_touched`, `the_ownership_recovery_names_a_repair_for_every_unowned_input`; integration `ship_readiness_names_unowned_production_source_before_finalize_and_the_recovery_ships` |
| REQ-change-053 | unit `the_ownership_recovery_narrows_a_scope_over_files_the_change_never_touched`, `the_ownership_recovery_quotes_every_interpolated_word` |
