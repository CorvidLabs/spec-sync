---
change: make-check-the-product-and-stop-change-check-from-spawning-project-tests
artifact: testing
---

# Testing

## Discriminators

- `change_check_does_not_execute_configured_project_commands` — python sentinel file must not appear.
- `change_check_does_not_spawn_a_configured_verification_child` — reporter script never writes `group.txt`.
- `change_check_does_not_wait_on_a_held_cargo_build_lock` — held Cargo locks are not named.
- `check_does_not_walk_sdd_when_policy_is_on` — `specsync check` stays silent on a corrupt workspace.

## Control

- `change_check_fails_when_specs_and_code_drift` — phantom export `does_not_exist` fails.
- `failed_spec_sync_is_retryable_with_append_only_history` — failed then passing spec sync keeps attempts.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-023 | `src/change_tests.rs` `failed_spec_sync_is_retryable_with_append_only_history` |
| REQ-change-049 | `src/change_tests.rs` `change_check_does_not_execute_configured_project_commands`, `change_check_fails_when_specs_and_code_drift` |
| REQ-change-050 | `src/commands/init.rs` `fresh_init_leaves_sdd_off_so_the_first_check_is_just_drift` |
| REQ-change-058 | `src/change.rs` `evaluate_spec_code_sync`; `check_project_with_command_output` no longer loops `run_configured_command` |
| REQ-change-091 | `tests/integration/change.rs` `change_check_does_not_wait_on_a_held_cargo_build_lock`, `change_check_does_not_spawn_a_configured_verification_child` |
| REQ-cmd-check-004 | `tests/integration/commands.rs` `check_does_not_walk_sdd_when_policy_is_on` |
| REQ-cmd-init-005 | `src/commands/init.rs` `fresh_init_leaves_sdd_off_so_the_first_check_is_just_drift` |
| REQ-agents-check-audit-commands-001 | `src/agents.rs` `install_claude_creates_skill_and_command`; generated check skill says spec↔code sync |
