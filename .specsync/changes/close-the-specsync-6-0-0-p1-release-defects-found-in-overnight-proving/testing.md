---
change: close-the-specsync-6-0-0-p1-release-defects-found-in-overnight-proving
artifact: testing
---

# Testing

Unit and integration tests for each P1, plus the existing `fix_`, `generate_`, `new_`, and `get_spec_symbols` regressions.

## Requirement evidence

| ID | Evidence |
|----|----------|
| REQ-change-097 | `deleting_verification_attempts_refuses_adopted_finalization`, `emptying_verification_attempts_refuses_adopted_finalization`, `verifying_change_refuses_to_recreate_a_missing_attempts_ledger` |
| REQ-change-098 | `canonical_definition_payload_folds_crlf_to_lf`, `canonical_definition_payload_preserves_a_lone_carriage_return`, `canonical_tasks_payload_folds_crlf_before_checkbox_rewrite` |
| REQ-change-099 | `bundled_lifecycle_limits_match_the_github_scripts_copy` |
| REQ-cli-args-017 | `check_accepts_repeatable_spec_flag`, `check_repeatable_spec_flag_filters_the_same_as_positional` |
| REQ-cli-011 | `check_accepts_repeatable_spec_flag`, `check_repeatable_spec_flag_filters_the_same_as_positional` |
| REQ-cmd-check-016 | `fix_ignores_fenced_exported_heading_before_the_table` |
| REQ-cmd-generate-002 | `generate_fails_closed_when_unspecced_modules_have_no_source_files` |
| REQ-cmd-new-001 | `new_auto_detects_single_source_file` |
| REQ-cmd-new-004 | `new_refuses_reserved_and_invalid_module_names` |
| REQ-cmd-scaffold-004 | `scaffold_dir_must_stay_beneath_the_project_root` |
| REQ-config-014 | `test_load_config_malformed_json_sets_load_error`, `test_load_config_malformed_v4_json_sets_load_error`, `malformed_json_config_refuses_rules_rehash_compact_and_view` |
| REQ-generator-005 | `confined_generation_path_rejects_escape_and_absolute`, `generate_fails_closed_when_unspecced_modules_have_no_source_files` |
| REQ-git-utils-005 | `git_inherited_env_excludes_secrets_and_git_overrides`, `git_ceiling_stops_walk_into_a_host_worktree` |
| REQ-parser-004 | `fenced_example_backticks_are_not_documented_exports`, `fenced_exported_heading_is_not_a_public_api_subsection` |
| REQ-mcp-008 | `git_inherited_env_excludes_secrets_and_git_overrides`, `git_ceiling_stops_walk_into_a_host_worktree` |
