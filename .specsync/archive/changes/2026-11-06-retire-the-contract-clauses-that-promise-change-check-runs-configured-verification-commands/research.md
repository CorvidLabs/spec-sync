---
change: retire-the-contract-clauses-that-promise-change-check-runs-configured-verification-commands
artifact: research
---

# Research

Call sites at origin/main ad50bdef, so every reworded clause states what the code does:

- `evaluate_spec_code_sync` (src/change/sdd_version.rs) compares this change's specs to source
  in-process. Its doc comment says it does not spawn a process and that `verification_commands`
  are not executed.
- `run_configured_command`, `reject_direct_lifecycle_verification` and `cargo_run_targets_specsync`
  (src/change/validate_loaded_change.rs) are `#[allow(dead_code)]`. Only unit tests call them.
  `run_configured_command` is the only code that sets `SPECSYNC_VERIFICATION_CONTEXT` on a child.
- `verification_commands_for_change` (routing) is `#[cfg_attr(not(test), allow(dead_code))]`. Only
  unit tests call it.
- The live guard is `reject_recursive_lifecycle_dispatch` / `verification_recursion_error` in
  src/main.rs. A specsync process that inherits `SPECSYNC_VERIFICATION_CONTEXT` exits 1 once, naming
  the parent. It is covered by `indirect_recursive_lifecycle_check_fails_once_with_context` and
  `indirect_recursive_lifecycle_subcommands_fail_once_with_context`.
- `change status --json` reports `summary.verification_commands` as the scoped
  `specsync check --spec ...` command only.
- `grep -rn check_project_quiet src/` returns nothing. The quiet reporting mode that REQ-change-015
  required is gone, as REQ-change-058 requires.
- `comment` reports spec-check results only (REQ-cmd-comment-004, src/commands/comment.rs).
  `neither_comment_nor_change_check_spawns_configured_project_commands` configures
  `rustc --version --verbose` and asserts that neither command prints its output.
- The bootstrap digest (`policy_enforcement_projection`, src/change/collect_requirement_ids.rs) clears
  `verification_commands` before hashing. `init` writes an empty list and prints no instruction to
  fill it in (no `verification` string in src/commands/init.rs).

The REQ-change-091 discriminators (`change_check_does_not_spawn_a_configured_verification_child`,
`change_check_does_not_wait_on_a_held_cargo_build_lock`) already pin the runtime. What was missing is
a test that fails when a contract goes back to promising the opposite.
