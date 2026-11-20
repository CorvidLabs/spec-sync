---
change: keep-specsync-s-own-caches-and-journal-outside-the-work-tree
artifact: testing
---

# Testing

Fixtures are temporary directories. Every link points inside the fixture's own temporary directory, and "outside" is a sibling directory there. Integration tests set `XDG_CACHE_HOME` to a folder in the fixture, so nothing is written to the user's cache.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-util-006 | `src/util/local_state.rs`: `local_state_lives_outside_the_work_tree`, `local_state_refuses_links_fifos_and_oversized_files` |
| REQ-util-007 | `tests/integration/project_writes.rs`: `an_untracked_project_inside_another_work_tree_is_confined_to_itself`, `a_subproject_writes_through_a_link_that_stays_inside_its_repository`, `a_subproject_reads_specs_through_a_link_that_stays_inside_its_repository`, `a_subproject_link_that_leaves_its_repository_is_still_refused` |
| REQ-util-008 | `tests/integration/project_reads.rs`: `the_scan_lexer_blanks_only_comments_and_literals`, `no_project_file_is_read_outside_the_bounded_read`; `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers`, `no_spawned_git_command_writes_files_unmarked` |
| REQ-hash-cache-009 | `src/hash_cache.rs`: `a_forged_pass_an_unpacked_tree_ships_is_never_replayed`, `a_forged_pass_committed_in_another_letter_case_is_never_replayed`, `a_forged_pass_in_a_cache_git_tracks_is_never_replayed`, `a_link_at_the_work_tree_cache_path_is_never_followed`, `save_creates_nothing_through_a_dangling_link_or_a_linked_specsync`, `load_reads_no_cache_through_a_link` |
| REQ-cmd-check-021 | `tests/integration/repository_inputs.rs`: `check_never_uses_a_hash_cache_an_unpacked_tree_ships`, `check_never_uses_a_hash_cache_git_tracks`; `tests/integration/check.rs`: `check_never_writes_the_hash_cache_through_a_link`; `tests/integration/config.rs`: `check_creates_hash_cache` |
| REQ-rehash-003 | `tests/integration/check.rs`: `rehash_never_writes_the_hash_cache_through_a_link`; `tests/integration/repository_inputs.rs`: `rehash_leaves_a_hash_cache_git_tracks` |
| REQ-change-162 | `src/change_tests/journal_replay_targets.rs`: `a_journal_in_the_work_tree_is_never_replayed`, `a_running_transaction_keeps_its_journal_outside_the_work_tree`, `a_journal_git_tracks_is_never_replayed`, `a_journal_never_rewrites_or_deletes_git_files`; `tests/integration/repository_inputs.rs`: `change_new_never_replays_a_journal_an_unpacked_tree_ships`; `tests/integration/project_reads.rs`: `a_lifecycle_write_does_not_wait_on_or_follow_the_transaction_journal`, `change_new_never_replays_a_committed_journal_into_git` |
| REQ-cmd-resolve-002 | `src/commands/resolve.rs`: `spec_cache_never_returns_an_entry_the_work_tree_holds`, `spec_cache_never_writes_or_reads_through_a_link`, `spec_cache_creates_nothing_through_a_dangling_link_or_a_linked_directory`, `test_spec_cache_roundtrip` |
| REQ-cmd-resolve-003 | `tests/integration/project_reads.rs`: `no_project_file_is_read_outside_the_bounded_read` |
| REQ-hooks-009 | `tests/integration/project_writes.rs`: `hooks_install_never_writes_through_a_linked_folder_on_the_hooks_path`, `hooks_install_never_creates_the_target_of_a_dangling_pre_commit_link`, `hooks_install_installs_into_a_hooks_folder_in_the_work_tree`; `src/hooks.rs`: `install_and_uninstall_precommit_refuse_symlinked_hook` |
| REQ-validator-052 | `tests/integration/project_reads.rs`: `no_project_file_is_read_outside_the_bounded_read` |

Red first on `a2a50824` with only the tests commit added (`b2c70e1f`): all 5 new unit tests and 8 of the 13 new integration tests failed. The read scan found 12 unmarked reads, `src/validator/tests.rs:3452` among them, the write scan 7 and the Git scan 5. The 5 that passed there are as they should be: the three monorepo controls, a real in-tree hooks folder, and the lexer test. The `src/util/local_state.rs` tests call the module this change adds, so they could not run on the base. What they cover is also driven by the CLI tests above.

Then the full verify lane and the Linux container run.
