# Lesson bundle: retire-the-contract-clauses-that-promise-change-check-runs-configured-verification-commands

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Retire the contract clauses that promise change check runs configured verification commands
- **Kind**: Documentation
- **Specs**: change, cmd_comment
- **Paths**: specs/change/requirements.md, specs/change/change.spec.md, specs/change/testing.md, specs/change/change.3md, specs/cmd_comment/requirements.md, specs/cmd_comment/cmd_comment.spec.md, specs/cmd_comment/context.md, specs/cmd_comment/cmd_comment.3md, site/src/content/docs/why-specsync.md, site/src/content/docs/comparisons/adversarial-proof.md, tests/integration/change.rs, tests/integration/comment.rs, tests/integration/helpers.rs, CHANGELOG.md, hi/ship.md, INTENT.md, src/change_tests/semantic_application_resolves_registry_backed_canonical_paths.rs, src/change_tests/canonical_resolution_tolerates_inert_legacy_registry_stub.rs
- **Acceptance**: SHIP-1.c.1: REQ-change-015 is retired, and REQ-change-019, REQ-change-027, REQ-change-030, REQ-change-061 and REQ-change-069 say no configured verification command is executed and cite REQ-change-023; the change spec, testing.md, context.md and change.3md keep no clause that promises execution.
- **Acceptance**: SHIP-1.c.1: REQ-cmd-comment-001 and REQ-cmd-comment-002 are retired because REQ-cmd-comment-004 reversed them; REQ-cmd-comment-003, the top-level acceptance list and the spec scenario say comment executes no configured SDD verification command; context.md and cmd_comment.3md no longer say change check streams one.
- **Acceptance**: SHIP-1.c.1: the why-specsync and adversarial-proof comparison pages no longer credit SpecSync with executing configured tests or isolating a verification shell, and say verification runs no configured command.
- **Acceptance**: SHIP-1.c.2: change_contracts_give_one_answer_on_configured_verification_commands, comment_contracts_say_no_configured_verification_command_runs and comparison_pages_do_not_credit_specsync_with_running_configured_tests fail on main and pass after the change.
- **Acceptance**: Runtime behavior is unchanged: no production source changes (only test files under src/change_tests and tests/integration), and the REQ-change-091 discriminators and neither_comment_nor_change_check_spawns_configured_project_commands still pass unmodified; REQ-change-019 and REQ-change-061 gain characterization tests whose mutation checks fail.

## Evidence

- Verification commit: `fb984546df498a137e870976b226c059a3914bb5`
- Base commit: `ad50bdef3db58a715877877d2407485bcaed0f32`
- Verified by: `specsync check --spec change --spec cmd_comment`

## From the change's context.md

# Context

The independent 2026-11-05 audit (finding D4, issue #848) found that the current SpecSync contracts
give two answers to one question. `specs/change/requirements.md` REQ-change-015 says reporting mode
"still executes every configured verification command", while REQ-change-023, REQ-change-049,
REQ-change-058 and REQ-change-091 say `change check` executes none, and the runtime agrees with the
latter. A deliberately wrong greeting passed `change check` and failed only the project's own test.
The audit asked for one current normative answer, and warned against fixing it by quietly adding test
execution to a structural command.

A sweep for the same promise found more clauses: REQ-change-019, REQ-change-027, REQ-change-030,
REQ-change-061 and REQ-change-069 in the change module; the top-level acceptance list,
REQ-cmd-comment-003 and one behavioral scenario in cmd_comment, plus a status line saying
`change check` streams the configured output; and three rows on two public comparison pages.
REQ-cmd-comment-001 and REQ-cmd-comment-002 contradict REQ-cmd-comment-004 for the same root cause:
#543 cut `comment` off from the lifecycle path that ran these commands.

Leif's decisions (2026-11-05 20:24 MST, relayed by orc t18): the intent was approved as drafted
(SHIP-1.c.1, SHIP-1.c.2). On a to e he accepted all: retire REQ-cmd-comment-001 and 002 here, reword
the site rows here, file the configured-commands warning as a follow-up (filed as #893), keep the dead
helpers and qualify their contracts, and leave `init --help` unchanged. No runtime behavior changes.

Ruled out: making `change check` run the commands (that reverses SHIP-1.c and REQ-change-091); deleting
the dead runner, classifier and routing helpers (decision d); adding a warning (decision c, #893).

## From the change's design.md

# Design

One rule, REQ-change-023: `change check` does not execute `.specsync/sdd.json`
`verification_commands`. Every other clause either defers to it by ID or is retired.

| Clause | Was | Becomes | Why |
|---|---|---|---|
| REQ-change-015 | quiet reporting mode that executes every configured command | REMOVED | REQ-change-058 says that mode SHALL NOT exist |
| REQ-change-019 AC3 | "a failed configured command" is one of three distinct diagnostics | the two evidence diagnostics stay distinct and neither names a command | no command runs, so none can fail |
| REQ-change-027 | configured verification rejects re-entry; native commands "execute once" | a lifecycle command refuses an inherited context; SpecSync sets it on no child | only the inherited-marker guard is live |
| REQ-change-030 | preserves "native verification commands"; Cargo identity criteria read as live | body drops it; Cargo criteria describe the retained classifier no command consults | the classifier is dead code, kept by decision d |
| REQ-change-061 AC2 | "populating them as initialization instructs" | "editing the list"; init writes it empty | init gives no such instruction |
| REQ-change-069 | commands "a change receives" | commands the retained routing helper selects; none executed | routing is test-only |
| REQ-cmd-comment-001, 002 | comments fold SDD lifecycle failures | REMOVED | REQ-cmd-comment-004 reverses them (#543) |
| REQ-cmd-comment-003 AC1 | command output captured away from stdout | comment executes no configured command | nothing to capture |
| cmd_comment scenario | the command still executes | the command is not executed | same |
| cmd_comment top-level AC, context.md and 3md status | still execute; change check streams it | executes none; neither spawns it | matches the integration test |
| site comparison rows | SpecSync runs configured tests; no-shell verification | your CI runs the tests; verification runs no configured command | public guidance makes the same promise |

Requirement edits go through the semantic deltas, so `change check` materializes them and the
stale-MODIFIED gate protects them. Companion prose (the top-level list in cmd_comment
requirements.md, context.md, testing.md and lessons) and site pages are direct edits inside the
declared paths. History stays as it is: change-log rows and earlier lesson planes describe what
was true then.

## From the change's testing.md

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

## Where these lessons go

- `specs/change/change.3md`
- `specs/cmd_comment/cmd_comment.3md`
