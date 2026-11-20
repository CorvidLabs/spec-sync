---
change: walk-configured-folders-only-through-the-confined-walk-and-keep-the-read-cache-in-the-user-cache-folder
artifact: testing
---

# Testing

Fixtures are temporary directories, with "outside" a sibling folder. Integration tests set `XDG_CACHE_HOME` outside every fixture.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-util-009 | `src/change_tests/fast_archived_reads.rs`: `the_read_cache_never_lives_in_the_git_directory`, `a_second_worktree_reads_warm_from_the_shared_cache` |
| REQ-util-010 | `src/schema.rs`: `a_schema_folder_linked_out_of_the_project_is_never_listed`; `src/validator/tests.rs`: `a_missing_file_suggestion_never_lists_a_folder_outside_the_project`; `tests/integration/project_writes.rs`: `scaffold_never_lists_a_specs_folder_linked_out_of_the_project` |
| REQ-change-164 | `src/change_tests/journal_replay_targets.rs`: `requirement_id_proposals_never_walk_a_specs_link_that_leaves_the_project`; `src/change_tests/fast_archived_reads.rs`: `the_read_cache_never_lives_in_the_git_directory` |
| REQ-validator-053 | `src/validator/requirement_ids.rs`: `duplicate_requirement_ids_are_never_looked_for_through_a_specs_link_out`, `an_unreadable_specs_subdirectory_is_an_error_naming_the_path`; `src/validator/tests.rs`: `a_missing_file_suggestion_never_lists_a_folder_outside_the_project` |
| REQ-config-018 | `src/config.rs`: `source_detection_never_follows_a_linked_folder` |
| REQ-schema-003 | `src/schema.rs`: `a_schema_folder_linked_out_of_the_project_is_never_listed` |
| REQ-commands-016 | `src/commands/mod.rs`: `case_collision_detects_case_fold_duplicates`; `tests/integration/project_writes.rs`: `scaffold_never_lists_a_specs_folder_linked_out_of_the_project` |
| REQ-cmd-scaffold-008 | `tests/integration/project_writes.rs`: `scaffold_never_lists_a_specs_folder_linked_out_of_the_project` |
| REQ-registry-005 | `src/registry.rs`: the existing `generate_registry` tests, on the confined walk |
| REQ-cmd-wizard-004 | `src/commands/wizard.rs`: the existing wizard tests, on the confined walk |
| REQ-mcp-014 | `tests/integration/mcp.rs`: the existing `specsync_issues` tests, on the confined walk |
| REQ-cmd-check-022 | `tests/integration/config.rs` and `tests/integration/check.rs`: the schema-input cache tests, on the confined listing |

Red first on `4516f355` with only change 8's records and tests added (`38cc5e87`): all 6 new unit tests and the new scaffold CLI test failed. `change adopt`'s proposals listed the two `requirements.md` files outside, the duplicate-ID check named `specs/alpha` and `specs/beta`, source detection found `lib` outside, the schema snapshot named `outside_only.sql`, the suggestion listed `../outside/data`, `scaffold` refused `auth` because of `Auth` outside, and the read cache was under `.git`.

Then the full verify lane and the Linux container run.
