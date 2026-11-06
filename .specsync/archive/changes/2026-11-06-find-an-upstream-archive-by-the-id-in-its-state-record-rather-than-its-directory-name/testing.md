---
change: find-an-upstream-archive-by-the-id-in-its-state-record-rather-than-its-directory-name
artifact: testing
---

# Testing

`src/change_tests/upstream_archive.rs::upstream_archive_is_found_by_its_state_record_not_its_directory_name` fails on main cd8cab5e ("a renamed upstream package must still be found by its state record"). `upstream_archive_is_reported_only_when_head_does_not_contain_it`, the `archived_upstream` integration tests and `lifecycle_commands_stay_within_git_spawn_ceilings` must stay green.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-139 | `upstream_archive_is_found_by_its_state_record_not_its_directory_name` (renamed package found by its state record), `upstream_archive_is_reported_only_when_head_does_not_contain_it`, `archived_upstream` integration tests, `lifecycle_commands_stay_within_git_spawn_ceilings` (bounded Git processes) |
