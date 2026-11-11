# Lesson bundle: answer-and-actor-validation-round-five-recoveries-run-as-printed-exact-none-every-actor-verb-tested

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Answer and actor validation round five: recoveries run as printed, exact none, every actor verb tested
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/sdd_version.rs, src/change/validate_loaded_change.rs, src/change/retirement.rs, src/change_tests/approve_ownership_names_the_spec.rs, src/change_tests/approve_rejects_a_declared_path_owned_by_an_undeclared_module.rs, src/change_tests/recovery_retirement.rs, src/change_tests/historical_reconciliation.rs, tests/integration/change.rs, tests/integration/doctor.rs, site/src/content/docs/cli.md
- **Acceptance**: owner recovery scaffolds a configured owner with no spec before declaring it; empty constraints keep their none guidance; an exact none is refused as ambiguous when something is named none; registered-but-missing specs and unreadable registries get their own messages; zero-width and directional-mark characters are refused in actors; reopen, retire, correct, correct-owner, reconcile-history and the doctor late close each have a bad-actor test; REQ-change-150, invariant 56 and the lesson match

## Evidence

- Verification commit: `768027bfd088235eb7256de53839306923746b71`
- Base commit: `1c07640aa5960c97c0d7fe9b7edb49aa42b8556b`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

PR #1026 round 5. The second independent review, at `6c579737`, approved at about 85% with should-fixes:

- Owner recovery printed `answer affected_specs core` for a configured owner with no spec, which the round-3 existence check refuses.
- Empty `constraints` lost its `none` hint.
- Registered-but-missing specs and unreadable registries got misleading or no messages.
- `change new --spec` and the bootstrap exemption were not in the spec text.
- No tests covered the other actor verbs.
- `none` could collide with a real id or module.
- U+200B, U+FEFF and U+200E/F were accepted.

All are fixed here. The review found no data-loss or ledger-corruption path. Leif's standing bug-fix rule applies, and REQ-change-150 is this PR's own requirement.

## From the change's design.md

# Design

`declare_owners_answer` checks each newly declared owner's canonical spec. Owners without one
are named with `specsync new <module>` before the answer, so every printed command runs as
printed once the earlier ones have. `clearable_list_answer` takes an exact `none` and refuses it
when a change id (dependencies) or a known module (affected_specs) is itself `none`. The answer
`none,` names it instead. `ensure_affected_specs_exist` loads the registry first and reports a
load error. A registered module whose file is missing gets its own message. `normalize_actor`
also refuses U+200B to U+200F, U+2060 to U+2069 and U+FEFF.

## From the change's testing.md

# Testing

Unit and integration tests on temp projects.

After main brought in #1037's split of the ship-preflight recovery (modified inputs declare, untouched ones
narrow), the mixed declare-and-narrow recovery carries the same no-spec fallback as the declare-only one:
`the_mixed_recovery_names_a_declared_module_whose_spec_is_missing` fails with main's earlier mixed fallback.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-150 | unit `ownership_recovery_scaffolds_a_configured_owner_with_no_spec_before_declaring_it`, `the_mixed_recovery_names_a_declared_module_whose_spec_is_missing`, `refused_reopen_restores_the_archived_package`, `batch_owner_corrections_append_transactionally_or_not_at_all`; unit `ownership_recovery_names_every_listed_module_that_has_no_spec`, `the_owner_hint_and_the_printed_answer_agree_in_a_project_with_no_specs`, `a_spec_behind_a_symlink_or_a_case_variant_is_not_the_modules_spec`, `a_spelling_difference_above_the_module_or_in_normalization_is_not_a_case_variant`, `recovery_retirement_resumes_with_the_actor_its_receipt_recorded`, `the_actor_rule_refuses_every_invisible_character_but_zwnj_and_zwj`; integration `change_answer_refuses_empty_lists_and_none_clears_optional_lists` (yes/no choices, `affected_paths []`, `None`), `change_approve_trims_and_validates_actor_identity`, doctor late-close tests |

## Where these lessons go

- `specs/change/change.3md`
