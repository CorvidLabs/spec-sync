# Lesson bundle — show-an-opened-change-the-lessons-and-principles-it-should-honor-and-require-its-plan-to-say-how

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Show an opened change the lessons and principles it should honor, and require its plan to say how
- **Kind**: Feature
- **Specs**: change, cmd_change, cmd_rules
- **Paths**: src/change.rs, src/change_tests.rs, src/commands/change.rs, src/commands/rules.rs, specs/change/change.spec.md, specs/change/requirements.md, specs/change/context.md, specs/change/tasks.md, specs/change/testing.md, specs/cmd_change/cmd_change.spec.md, specs/cmd_change/requirements.md, specs/cmd_change/context.md, specs/cmd_change/tasks.md, specs/cmd_change/testing.md, specs/cmd_rules/cmd_rules.spec.md, specs/cmd_rules/requirements.md, specs/cmd_rules/context.md, specs/cmd_rules/tasks.md, specs/cmd_rules/testing.md, site/src/content/docs/configuration.md, site/src/content/docs/workflow.md, docs/HLD.md, hi/plan.md
- **Acceptance**: When I open a change, I can see the lessons already written for the modules it touches, and the project's principles when I have any.
- **Acceptance**: My agent sees the same thing in the structured result.
- **Acceptance**: A module that still has only its original scaffold is not presented as if it had learned something.
- **Acceptance**: A change that needs a plan cannot be approved until that plan says what I will do, what I will not do, the steps, the risks, and which existing constraints I am honoring.
- **Acceptance**: A change the interview did not select a plan for still does not grow one.
- **Acceptance**: While a change is still a draft, and only when lessons or principles exist, I am asked which of them constrain it, and "none" is a complete answer.
- **Acceptance**: Changing that answer is a change of the agreed scope.
- **Acceptance**: When a change is archived I am shown where its lesson bundle is, and I can still merge without copying those lessons into the spec.
- **Acceptance**: A lesson becomes a standing rule only when I write it into the project's principles, and the next change shows me that file before I approve it.

## Evidence

- Verification commit: `4c3c6783b6ff040e638b78638aa87f26f48e5d91`
- Base commit: `684aed366163c29ce3b5df22b3c4716353d93fcf`
- Verified by: `specsync check --spec change --spec cmd_change --spec cmd_rules`

## From the change's context.md

# Context

Opening a change today can print a line count for `specs/<module>/context.md`, and only in human text. `change new --json` skips that pointer. `finalize` and `ship` already put `lesson_bundle` in JSON. Their text next action is only the merge instruction, and a test locks that wording so writing lessons cannot withhold a merge.

`next_questions` does not know about the working tree. The approval gate and the status line both call it. A constraints question has to be computed with the project root, and only while the change is still a draft, or every historical approval becomes an unfinished interview.

`ApprovedScopeV1.answers` is already the whole answer map. A `constraints` entry changes the scope digest for the change that records it. Older records omit the key, so their digest stays the same.

A selected plan is complete today once its placeholder comment is gone. Headings alone would pass. The new check has to require substantive text under Approach, Out of scope, Steps, Risks, and Constraints consulted, and the status list of incomplete artifacts has to use that same check.

`specsync rules` loads spec-shape configuration only. `principles_file` is hashed into the execution digest and never printed.

Do not shell out to a model. Do not fail `change new` because a context file cannot be read. Do not treat the generated context scaffold as a lesson.

## From the change's design.md

# Design

The briefing is computed, not stored. It is not a field on `ChangeRecord` and not a field on `ApprovedScopeV1`.

```text
OpenChangeBriefing
  lessons: [{ path, lines, excerpt, truncated }]
  principles: none | { path, excerpt, truncated, missing }
```

`accumulated_lessons` stays the scaffold filter. The briefing uses that filter, then keeps a bounded prefix of the substantive lines. Twenty lines or 1,200 characters, whichever comes first, is enough to see the lesson. `truncated` is true when something was left in the file.

`principles` is present when `.specsync/sdd.json` names `principles_file`. `missing` is true when that path cannot be read. The command still succeeds.

The constraints question is last. It is absent unless the change is a draft, the other interview questions are answered, the `constraints` key is missing, and either the briefing has a lesson or a principles file is configured. Re-answering `affected_specs` deletes the key.

Plan completeness is a second predicate on top of the TODO check, applied only to `ArtifactKind::Plan`. A section body counts when it has a line that is not blank, not an HTML comment, and not a placeholder TODO. The five headings are matched case-insensitively: Approach, Out of scope, Steps, Risks, Constraints consulted.

`specsync rules` prints a "Standing principles" block after the built-in rules. It reads the SDD policy and writes nothing.

## From the change's testing.md

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

## Where these lessons go

- `specs/change/context.md`
- `specs/cmd_change/context.md`
- `specs/cmd_rules/context.md`
