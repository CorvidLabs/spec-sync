---
change: print-a-single-choice-interview-answer-literally-in-the-approval-refusal-so-the-named-change-answer-command-runs-as
artifact: testing
---

# Testing

Failing first on main db641967, implementation unchanged (Chough OBS-product-test-41c82ca012cdb82a78af57e7):

- `a_single_choice_question_names_its_choice_literally`: the refusal prints `constraints <none>`.
- `a_draft_without_specs_or_paths_is_refused_for_the_question_status_asks_first`: approve names `affected_paths` while status asks `affected_specs` first.
- `approve_refusals_after_the_landing_page_change_new_name_the_command_that_finishes_them` and `approve_refusals_for_a_draft_created_without_scope_name_the_scope_answers`: the harness no longer rewrites `<none>`, and after each refusal text status must name the same command; it names `acceptance_criteria <answer>` (JSON) or the artifacts step (text).

After the change all of these pass with the other hint tests, the documented-workflow tests, `fledge lanes run verify` and `specsync check --strict`.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-133 | `a_single_choice_question_names_its_choice_literally`, `a_draft_without_specs_or_paths_is_refused_for_the_question_status_asks_first`, `a_draft_without_affected_paths_names_the_affected_paths_answer`, `a_draft_without_affected_specs_names_the_affected_specs_answer`, `a_record_past_draft_is_never_told_to_run_change_answer`, `approve_refusals_for_a_draft_created_without_scope_name_the_scope_answers` |
| REQ-cmd-change-answer-hint-001 | `approve_refusals_after_the_landing_page_change_new_name_the_command_that_finishes_them`, `approve_refusals_for_a_draft_created_without_scope_name_the_scope_answers` |
