---
change: keep-specsync-s-own-state-only-in-the-user-cache-folder
artifact: testing
---

# Testing

Fixtures are temporary directories. "Outside" is a sibling directory there. Integration tests set `XDG_CACHE_HOME` to a folder in the system temporary directory, outside every fixture project.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-util-006 | `src/util/local_state.rs`: `local_state_never_lives_in_a_git_directory`, `local_state_lives_outside_the_work_tree`, `local_state_refuses_links_fifos_and_oversized_files`; `tests/integration/repository_inputs.rs`: `check_never_uses_a_hash_cache_an_archive_ships_in_its_git_directory`, `check_refuses_a_user_cache_folder_inside_the_project` |
| REQ-util-008 | `tests/integration/project_reads.rs`: `no_project_file_is_read_outside_the_bounded_read`; `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers` |
| REQ-git-utils-012 | `tests/integration/project_writes.rs`: `a_project_folder_named_like_a_glob_is_still_confined_to_itself`, `an_untracked_project_inside_another_work_tree_is_confined_to_itself`, `a_subproject_writes_through_a_link_that_stays_inside_its_repository` |
| REQ-change-162 | `src/change_tests/journal_replay_targets.rs`: `a_journal_under_the_git_directory_is_never_replayed`, `a_registry_entry_that_is_not_a_spec_never_opens_a_replay`, `a_registry_mapped_spec_is_recovered_and_its_neighbours_are_not`, `a_journal_in_the_work_tree_is_never_replayed` |
| REQ-hooks-009 | `tests/integration/project_writes.rs`: `hooks_install_makes_no_folder_for_a_local_hooks_path_outside_the_work_tree`, `hooks_install_still_makes_a_hooks_folder_from_the_global_config`, `hooks_install_never_writes_through_a_linked_folder_on_the_hooks_path` |
| REQ-cmd-check-021 | `tests/integration/repository_inputs.rs`: `check_never_uses_a_hash_cache_an_archive_ships_in_its_git_directory`, `check_refuses_a_user_cache_folder_inside_the_project`, `check_never_uses_a_hash_cache_git_tracks` |

Red first on `e3d508ac` with only change 7's records and tests added (`3fcb6de7`): all 3 new unit tests and 5 of the 7 new integration tests failed, and the read scan found 10 new unmarked opens. The 2 that passed are the global-config hooks control and the write scan, which has no new findings.

Then the full verify lane and the Linux container run.
