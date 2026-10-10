---
change: show-an-opened-change-the-lessons-and-principles-it-should-honor-and-require-its-plan-to-say-how
artifact: testing
---

# Testing

Evidence is unit tests in `src/change_tests.rs` and `src/commands/change.rs`, plus the rules command's existing module tests if present. `change check` compares specs to code and does not run the suite. The tests below are the evidence named for each requirement.

REQ-change-104
- `constraints_question_is_asked_only_for_a_draft_that_has_lessons_or_principles`
- `constraints_question_is_absent_when_the_context_is_only_a_scaffold`
- `answering_none_completes_the_constraints_question`
- `changing_affected_specs_asks_the_constraints_question_again`
- `a_constraints_answer_changes_the_scope_digest`

REQ-change-105
- `a_plan_of_empty_headings_is_incomplete`
- `a_plan_with_substantive_sections_is_complete`
- `optional_companions_are_selected_by_policy` already shows a documentation change does not select a plan. Extend it, or add `a_bug_fix_does_not_grow_a_plan`, so a kind that did not select a plan still does not.

REQ-change-106
- `open_change_briefing_quotes_substantive_lessons_and_omits_a_scaffold`
- `open_change_briefing_names_a_configured_principles_file`
- `open_change_briefing_survives_an_unreadable_context`

REQ-cmd-change-018
- `change_new_json_includes_the_briefing` in the command tests, or a domain test of the value the command prints
- `ship_does_not_gate_merge_on_writing_lessons` stays green
- `finalize_text_names_the_lesson_bundle_beside_the_merge_instruction`

REQ-cmd-rules-002
- `rules_names_the_principles_file_and_does_not_invent_one`
- `rules_says_when_no_principles_file_is_configured`

Manual check: `specsync change status` on this change, after the code lands, shows the change module's existing lessons and does not ask `constraints` again, because this draft already answered it.
