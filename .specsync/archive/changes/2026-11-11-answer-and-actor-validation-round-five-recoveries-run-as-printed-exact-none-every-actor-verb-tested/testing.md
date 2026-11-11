---
change: answer-and-actor-validation-round-five-recoveries-run-as-printed-exact-none-every-actor-verb-tested
artifact: testing
---

# Testing

Unit and integration tests on temp projects.

After main brought in #1037's split of the ship-preflight recovery (modified inputs declare, untouched ones
narrow), the mixed declare-and-narrow recovery carries the same no-spec fallback as the declare-only one:
`the_mixed_recovery_names_a_declared_module_whose_spec_is_missing` fails with main's earlier mixed fallback.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-150 | unit `ownership_recovery_scaffolds_a_configured_owner_with_no_spec_before_declaring_it`, `the_mixed_recovery_names_a_declared_module_whose_spec_is_missing`, `refused_reopen_restores_the_archived_package`, `batch_owner_corrections_append_transactionally_or_not_at_all`; unit `ownership_recovery_names_every_listed_module_that_has_no_spec`, `the_owner_hint_and_the_printed_answer_agree_in_a_project_with_no_specs`, `a_spec_behind_a_symlink_or_a_case_variant_is_not_the_modules_spec`, `a_spelling_difference_above_the_module_or_in_normalization_is_not_a_case_variant`, `recovery_retirement_resumes_with_the_actor_its_receipt_recorded`, `the_actor_rule_refuses_every_invisible_character_but_zwnj_and_zwj`; integration `change_answer_refuses_empty_lists_and_none_clears_optional_lists` (yes/no choices, `affected_paths []`, `None`), `change_approve_trims_and_validates_actor_identity`, doctor late-close tests |
