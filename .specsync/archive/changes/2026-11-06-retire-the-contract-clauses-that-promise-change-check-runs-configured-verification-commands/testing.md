---
change: retire-the-contract-clauses-that-promise-change-check-runs-configured-verification-commands
artifact: testing
---

# Testing

The runtime does not change, so the behavioral discriminators that already pin it stay as they
are and must still pass unmodified: `change_check_does_not_spawn_a_configured_verification_child`,
`change_check_does_not_wait_on_a_held_cargo_build_lock`,
`an_unheld_cargo_build_lock_is_not_reported_during_verification`,
`change_check_does_not_execute_configured_project_commands` and
`neither_comment_nor_change_check_spawns_configured_project_commands`.

What the audit found missing is a test that fails when a contract goes back to promising the
opposite. Three contract pins do that (SHIP-1.c.2). They read the tracked contracts, not the binary,
and say so in their honest labels. Each failed on main ad50bdef with the contracts unchanged
(`cargo test --test integration -- change_contracts_give_one_answer comparison_pages_do_not_credit
comment_contracts_say_no_configured`: 0 passed, 3 failed):

- `change_contracts_give_one_answer_on_configured_verification_commands`. REQ-change-015 must be
  absent, REQ-change-023 must state the rule, REQ-change-019/027/030/061/069 must cite it, and five
  retired phrases must be absent from the change spec, requirements, testing.md, context.md and 3md.
  On main it failed because REQ-change-015 was present.
- `comment_contracts_say_no_configured_verification_command_runs`. REQ-cmd-comment-003 must say that
  comment executes no configured command, REQ-cmd-comment-004 must be present, REQ-cmd-comment-001 and
  002 must be absent, and four retired phrases must be absent from the cmd_comment contracts. On main it
  failed at REQ-cmd-comment-003.
- `comparison_pages_do_not_credit_specsync_with_running_configured_tests`. The three site rows must be
  gone, and both pages must say that verification runs no configured command. On main it failed at
  "Configured tests executed by CI gate".

Two reworded criteria had no dedicated test, so they gain characterization tests. Both pass on main
because the behavior does not change. Each failed under a temporary mutation of the code it guards,
and the source was restored:

- `missing_requirement_evidence_is_distinct_from_missing_semantic_evidence` (REQ-change-019 AC3). It
  fails when `evidence_gap_detail` returns the semantic-evidence message for both gaps.
- `editing_only_the_verification_command_list_keeps_the_bootstrap_exemption` (REQ-change-061 AC2). It
  fails when `policy_enforcement_projection` keeps `verification_commands` in the digest. Its control
  (editing `meaningful_paths`) must revoke the exemption.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-019 | `section_only_semantic_delta_can_satisfy_acceptance_evidence`, `missing_semantic_acceptance_evidence_is_not_reported_as_command_failure`, `missing_requirement_evidence_is_distinct_from_missing_semantic_evidence` (new, mutation-checked), `change_contracts_give_one_answer_on_configured_verification_commands` (contract pin) |
| REQ-change-027 | `indirect_recursive_lifecycle_check_fails_once_with_context`, `indirect_recursive_lifecycle_subcommands_fail_once_with_context` (one diagnostic naming the parent, no validation output), `change_contracts_give_one_answer_on_configured_verification_commands` (contract pin) |
| REQ-change-030 | `generated_sequence_scope_does_not_suppress_delivery_scope_question`, `semantic_application_resolves_registry_backed_canonical_paths`, `disabled_policy_skips_sequence_validation`, `change_check_does_not_execute_configured_project_commands`, `indirect_recursive_lifecycle_check_fails_once_with_context`, and for the retained classifier `native_cargo_check_argument_is_not_misclassified_as_specsync` and `cargo_manifest_path_detects_recursive_specsync_before_state_mutation` (they call the helper directly), `change_contracts_give_one_answer_on_configured_verification_commands` (contract pin) |
| REQ-change-061 | `adopt_re_pins_the_bootstrap_record_it_invalidates`, `editing_only_the_verification_command_list_keeps_the_bootstrap_exemption` (new, mutation-checked, with control), `change_contracts_give_one_answer_on_configured_verification_commands` (contract pin) |
| REQ-change-069 | `declaring_an_unrouted_module_never_reduces_verification`, `a_fully_routed_change_still_runs_only_its_component_commands`, `verification_routing_fails_closed_without_any_validator` (they call the routing helper directly), `change_check_does_not_spawn_a_configured_verification_child` (nothing routed is executed), `change_contracts_give_one_answer_on_configured_verification_commands` (contract pin) |
| REQ-cmd-comment-003 | `neither_comment_nor_change_check_spawns_configured_project_commands`, `ci_captures_comment_with_quiet_cargo_and_a_defensive_byte_cap`, `malformed_gradle_is_inconclusive_for_coverage_gating_commands`, `comment_contracts_say_no_configured_verification_command_runs` (contract pin) |

The REMOVED requirements (REQ-change-015, REQ-cmd-comment-001, REQ-cmd-comment-002) have no evidence
row by construction. Their absence is asserted by the contract pins.

## Acceptance criteria evidence

| Criterion | How it is proven |
| --- | --- |
| SHIP-1.c.1 change contracts | `change_contracts_give_one_answer_on_configured_verification_commands` |
| SHIP-1.c.1 cmd_comment contracts | `comment_contracts_say_no_configured_verification_command_runs` |
| SHIP-1.c.1 comparison pages | `comparison_pages_do_not_credit_specsync_with_running_configured_tests` |
| SHIP-1.c.2 pins fail on main, pass after | Failing run recorded above (Chough OBS-product-test-7d30046026e1f771b8545980); passing run after `change check` materializes the deltas |
| Runtime unchanged | `git diff origin/main --stat -- src` lists only `src/change_tests/*.rs`; the discriminators above pass unmodified in the full verify lane |

## Verification commands

- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (`fledge lanes run verify`)
- `specsync change check <id> --commit` (materializes the deltas and checks spec against code)
- `specsync check --strict --require-coverage 100 --force` (`fledge lanes run pre-push`)
- `cd site && bun test` for the edited site pages
