---
change: keep-approve-refusal-hints-to-drafts-cover-the-scope-questions-show-single-choice-placeholders-and-commit-the-quick
artifact: testing
---

# Testing

Failing first at 2b30841d (on #912's archive tip):

- `a_draft_without_affected_paths_names_the_affected_paths_answer` and `a_draft_without_affected_specs_names_the_affected_specs_answer`: the refusal names no command.
- `a_record_past_draft_is_never_told_to_run_change_answer`: an approved record missing its criterion is told to run `change answer`.
- `a_single_choice_question_shows_its_choice_as_the_placeholder`: `constraints` gets `"<answer>"`.
- `approve_refusals_for_a_draft_created_without_scope_name_the_scope_answers`: `at least one affected path is required` names no command.
- `landing_page_change_workflow_runs_from_its_quick_start`: at the CI-wait line the tree still has `?? .gitattributes`, `?? .gitignore` and `?? src/auth.ts`.

After the change all of these pass, along with the other documented-workflow tests, `fledge lanes run verify` on rust 1.89.0, and `specsync check --strict`.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-133 | `a_draft_without_affected_paths_names_the_affected_paths_answer`, `a_draft_without_affected_specs_names_the_affected_specs_answer`, `a_record_past_draft_is_never_told_to_run_change_answer`, `a_single_choice_question_shows_its_choice_as_the_placeholder`, `approve_refusals_for_a_draft_created_without_scope_name_the_scope_answers`, `approve_refusals_after_the_landing_page_change_new_name_the_command_that_finishes_them` |
