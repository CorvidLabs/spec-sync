---
change: close-the-low-items-left-after-the-hash-cache-advisory-red-tests-first
artifact: testing
---

# Testing

| Item | Test |
|------|------|
| W1 | `tests/integration/hardening_641.rs`: `archived_changes_linked_out_of_the_project_are_never_listed` |
| S1 | `hooks_install_writes_no_hook_into_an_existing_folder_the_repository_config_names` |
| S2 | `a_journal_left_for_an_earlier_project_at_the_same_path_is_not_replayed` |
| S3 | `correct_owner_does_not_wait_on_a_fifo_manifest`, `scaffold_does_not_wait_on_a_fifo_template` |
| S5 | `src/change_tests/hardening_641.rs`: `the_journal_is_not_kept_in_the_cache_folder` |
| L2 | `a_state_folder_others_can_write_is_refused` |
| X1 | `project_reads::no_project_file_is_read_outside_the_bounded_read`, `project_writes::no_spawned_git_command_writes_files_unmarked`, and the non-Git spawn scan |
| X2 | `a_hooks_path_spelled_through_a_link_to_the_work_tree_is_inside_it` (macOS) |
| H1 | `hooks_install_in_a_linked_worktree_makes_the_shared_hooks_folder` |
| N2 | `util::local_state::tests::a_project_that_would_hold_its_own_state_folder_is_refused` |
| C1 | `the_change_read_cache_is_never_waited_on` |
| C2 | `only_status_keeps_a_refused_spec_with_its_refusal` |
| C3 | `resolve_reports_a_refused_spec` |
| C4 | `stale_counts_a_refused_spec_as_unmeasurable` |
| C5 | `hash_cache::tests::save_refuses_a_cache_larger_than_the_load_bound` |
