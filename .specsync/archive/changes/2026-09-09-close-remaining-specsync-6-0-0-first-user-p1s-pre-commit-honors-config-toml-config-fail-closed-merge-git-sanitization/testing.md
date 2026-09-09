---
change: close-remaining-specsync-6-0-0-first-user-p1s-pre-commit-honors-config-toml-config-fail-closed-merge-git-sanitization
artifact: testing
---

# Testing

Fail-then-pass tests for each P1, plus the docs/next-action pins.

## Requirement evidence

| ID | Evidence |
|----|----------|
| REQ-hooks-002 | `install_precommit_creates_hook_file`, `install_precommit_uses_configured_hooks_path_and_preserves_exit_zero` (hook body has `specsync check` / `--root … check` without `--strict`) |
| REQ-hooks-003 | `init_add_spec_hooks_install_then_commit_succeeds_without_strict` |
| REQ-config-015 | `test_load_config_malformed_toml_sets_load_error`, `test_load_config_empty_toml_sets_load_error`, `test_load_config_directory_as_toml_sets_load_error`, `test_load_config_invalid_enforcement_enum_sets_load_error`, `malformed_toml_config_refuses_rules_rehash_compact_and_view`, `malformed_json_config_refuses_rules_rehash_compact_and_view`, `absent_config_still_runs_with_defaults` |
| REQ-git-utils-007 | `git_cmd_does_not_forward_sentinel_secrets` |
| REQ-merge-003 | `unmerged_paths_uses_sanitized_git_cmd`, `unmerged_paths_is_unknown_outside_a_git_repository` |
| REQ-change-100 | `workflow_v1_verifying_next_action_names_verify_accept_archive` |
| REQ-change-101 | `uncovered_paths_error_names_the_escape_hatch_and_ignore_precedence` (`--kind bug-fix`) |
