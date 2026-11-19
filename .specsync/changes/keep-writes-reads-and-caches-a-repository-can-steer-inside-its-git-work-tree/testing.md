---
change: keep-writes-reads-and-caches-a-repository-can-steer-inside-its-git-work-tree
artifact: testing
---

# Testing

Fixtures are temporary directories, most with their own `git init`. Every link points inside the fixture's own temporary directory, and "outside" is a sibling directory there.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-util-004 | `tests/integration/project_writes.rs`: `a_subproject_writes_through_a_link_that_stays_inside_its_repository`, `a_subproject_reads_specs_through_a_link_that_stays_inside_its_repository`, `a_subproject_link_that_leaves_its_repository_is_still_refused`, `an_absolute_link_is_refused_as_an_absolute_link`; `src/util.rs`: `writing_a_fifo_is_refused_without_blocking`, `reading_within_a_named_directory_stays_inside_it`, `removing_an_empty_directory_never_follows_a_link_out`, `finding_project_files_never_follows_a_link`, `finding_project_files_from_the_root_itself_lists_its_files` |
| REQ-util-005 | `tests/integration/project_writes.rs`: `hooks_install_never_writes_through_a_link_into_the_git_directory`, `change_answer_never_writes_change_md_into_a_hook`; `src/util.rs`: `project_file_functions_refuse_the_git_directory`, `project_file_functions_refuse_a_git_directory_under_any_name` |
| REQ-change-163 | `tests/integration/project_writes.rs`: `change_answer_never_writes_change_md_through_a_link_out`, `change_approve_never_writes_change_md_through_a_link_out`, `change_answer_never_creates_the_target_of_a_dangling_artifact_link`, `change_new_creates_nothing_in_a_changes_directory_linked_out`, `no_project_file_is_written_outside_the_confined_helpers` |
| REQ-cmd-check-020 | `tests/integration/project_writes.rs`: `check_fix_backup_copies_nothing_through_a_linked_backup_directory`, `check_fix_backup_never_copies_over_a_linked_backup_file`, `check_fix_backup_still_backs_up_a_project_with_no_link` |
| REQ-cmd-check-021, REQ-hash-cache-009 | `tests/integration/repository_inputs.rs`: `check_never_uses_a_hash_cache_git_tracks`, `rehash_refuses_a_hash_cache_git_tracks`; `src/hash_cache.rs`: `a_forged_pass_in_a_cache_git_tracks_is_never_replayed` |
| REQ-hash-cache-010 | `src/hash_cache.rs`: `prune_drops_keys_that_leave_the_project` |
| REQ-config-017 | `tests/integration/repository_inputs.rs`: `a_configured_directory_outside_the_project_is_a_config_error`; `tests/integration/mcp.rs`: `mcp_rejects_configured_read_and_write_path_escapes` |
| REQ-validator-050 | `tests/integration/repository_inputs.rs`: `a_specs_link_that_leaves_the_project_is_not_walked`, `spec_discovery_stops_at_the_depth_budget`; `src/validator/tests.rs`: `test_find_spec_files_empty_dir`, `test_find_spec_files_with_specs` |
| REQ-validator-051 | `tests/integration/project_reads.rs`: `no_project_file_is_read_outside_the_bounded_read` |
| REQ-changelog-002, REQ-git-utils-012, REQ-git-utils-013 | `tests/integration/repository_inputs.rs`: `changelog_skips_a_spec_blob_over_the_bound`; `src/git_utils.rs`: `capped_output_keeps_the_cap_and_reports_more` |
| REQ-cmd-import-004 | `tests/integration/repository_inputs.rs`: `import_from_dir_never_imports_through_a_link_out`, `import_from_an_outside_dir_stays_inside_that_dir` |
| REQ-cmd-init-registry-004 | `tests/integration/project_writes.rs`: `init_registry_creates_nothing_in_a_specsync_directory_linked_out` |
| REQ-agents-013 | `src/util.rs`: `removing_an_empty_directory_never_follows_a_link_out`; `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers` |
| REQ-archive-003 | `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers` |
| REQ-compact-003 | `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers` |
| REQ-merge-008 | `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers` |
| REQ-cmd-merge-006 | `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers` |
| REQ-cmd-lifecycle-006 | `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers` |
| REQ-cmd-init-010 | `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers` |
| REQ-generator-012 | `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers` |
| REQ-cmd-generate-005 | `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers` |
| REQ-cmd-issues-004 | `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers` |
| REQ-hooks-008 | `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers`; `hooks_install_never_writes_through_a_link_into_the_git_directory` |
| REQ-mcp-013 | `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers`; `tests/integration/mcp.rs`: `mcp_rejects_configured_read_and_write_path_escapes` |
| REQ-commands-015 | `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers` |
| REQ-rehash-002 | `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers` |
| REQ-cmd-view-003 | `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers` |
| REQ-deps-005 | `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers` |
| REQ-cmd-migrate-007 | `tests/integration/project_writes.rs`: `no_project_file_is_written_outside_the_confined_helpers` |

Red first on `1b3974e9` with only the tests commit added (`dfdaf4f6` in the builder's clone, `fc40682d` here): 22 of the 24 new integration tests and both new `hash_cache` unit tests failed. The write scan found 117 direct writes and the read scan 27 direct reads outside the helpers. The two controls passed there, as they should: a subproject link that leaves the repository is still refused, and a backup with no link still works. The new `src/util.rs` unit tests call helpers this change adds, so they could not run on the base; the Git-directory and walk cases they cover are also driven through the CLI by `hooks_install_never_writes_through_a_link_into_the_git_directory`, `change_answer_never_writes_change_md_into_a_hook` and `a_specs_link_that_leaves_the_project_is_not_walked`.

Then the full verify lane and the Linux container run.
