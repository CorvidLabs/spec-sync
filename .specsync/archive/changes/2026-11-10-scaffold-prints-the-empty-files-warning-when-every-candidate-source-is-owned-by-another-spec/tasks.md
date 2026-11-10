---
change: scaffold-prints-the-empty-files-warning-when-every-candidate-source-is-owned-by-another-spec
artifact: tasks
---

# Tasks

- [x] Share the empty-files warning between `new` (`cmd_add_spec`) and `scaffold`
- [x] `scaffold` prints the warning whenever it writes an empty `files:` list
- [x] Assert `files: []` and the warning in `scaffold_never_claims_a_file_another_spec_owns`, `scaffold_single_source_fallback_skips_an_owned_file` and `new_never_claims_an_owned_flat_file`
