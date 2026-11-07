# Lesson bundle: print-a-single-choice-interview-answer-literally-in-the-approval-refusal-so-the-named-change-answer-command-runs-as

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Print a single-choice interview answer literally in the approval refusal, so the named change answer command runs as printed (#958)
- **Kind**: BugFix
- **Specs**: change, cmd_change
- **Paths**: src/change/collect_requirement_ids.rs, src/change/historical_definition_digest_matches.rs, src/commands/change.rs, src/change_tests/approve_refusal_hints.rs, tests/integration/documented_workflows.rs, specs/change/, specs/cmd_change/, CHANGELOG.md, hi/docs.md, INTENT.md, .chough/
- **Acceptance**: The unfinished-interview refusal for a question with exactly one choice names that choice literally, e.g. `specsync change answer <id> constraints none`, so the command runs as printed (REQ-change-133, DOCS-3.a, proposed DOCS-3.b).
- **Acceptance**: A question with two or more choices keeps the `<a|b>` placeholder (`<yes|no>`), and one with no choices keeps `"<answer>"`.
- **Acceptance**: The documented-workflow probe runs the single-choice command exactly as printed: the harness no longer rewrites `<none>`.
- **Acceptance**: The approval guard refuses exactly the definitions it refused before; only the refusal text for a single-choice question changes.
- **Acceptance**: The changed unit test and the documented-workflow probe fail on main db641967 before the change and pass after it.

## Evidence

- Verification commit: `59dfd67b98f7d65ac4a20f15937ff4d9b4a3dea9`
- Base commit: `db6419678a19b5814e0b6c1ece58eb3543d8402e`
- Verified by: `specsync check --spec change --spec cmd_change`

## From the change's context.md

# Context

Issue #958, from the Codex review of #936: since #917 the unfinished-interview refusal shows a single choice as `<none>`, which Bash reads as an input redirection, so the printed `specsync change answer <id> constraints <none>` fails before SpecSync runs.

The issue says `none` is the only valid answer. It is not: `constraints` is free text with a `none` shortcut, asked only when lessons or principles exist (#917 answered it with a lesson citation). Leif chose option A (DOCS-3.b): print `none`, and keep telling the reader the lessons can be named instead.

While drafting, status was found to disagree with approve: JSON `next_action` printed `<answer>` for every question (even yes/no), text `change status` passed no questions to `text_mode_next_action` and so skipped `constraints` and named the artifacts step, and with both specs and paths missing approve asked for paths first while status asked for specs. Leif chose to fix status in the same change (DOCS-3.c, "2yes"), which adds the cmd_change spec.

The landing-page probe in `documented_workflows.rs` now runs the single-choice command exactly as printed and checks text and JSON status after every refusal.

## From the change's design.md

# Design

## One helper (`src/change/collect_requirement_ids.rs`)

`pub(crate) fn answer_command_hint(id, question, choices) -> String` returns the backticked command for one question:

- two or more choices: `<a|b>` (`<yes|no>`);
- exactly one choice: that choice itself (`constraints none`). For `constraints`, which is free text with a `none` shortcut, it appends `, or name the lessons or principles that constrain it instead`;
- no choice: the scope placeholders approve already uses (`acceptance_criteria "<observable outcome>"`, `affected_specs "<module>"`, `affected_paths "<path>"`), else `"<answer>"`.

`interview_incomplete_refusal` and `unfinished_definition_refusal` build their command with it, so the refusal text for every other question is byte-identical to today.

## Order (`validate_definition`)

The affected-specs check moves before the affected-paths check, so a draft missing both is refused for `affected_specs`, the question `next_questions` asks first. Each condition is unchanged, so the set of refused definitions is unchanged.

## Status

- JSON (`summarize_change`, `src/change/historical_definition_digest_matches.rs`): a draft with open questions gets `run <answer_command_hint(first question)>` instead of `run ... <question> <answer>`.
- Text (`text_mode_next_action`, `src/commands/change.rs`): the draft arm reads `change::next_questions_for(root, record)` itself, not the caller's list (which `change status` leaves empty, so text status skipped `constraints` and named the artifacts step while JSON named the answer), and names the same hint. The `questions` parameter becomes unused and is removed from its callers.

## Out of scope

The generic `<question> <answer>` lines in the handoff (`run ... for each remaining question`) and after a send-back name no particular question and stay as they are. `change answer --json` output (#839 / G-json) is not touched.

## From the change's testing.md

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

## Where these lessons go

- `specs/change/change.3md`
- `specs/cmd_change/cmd_change.3md`
