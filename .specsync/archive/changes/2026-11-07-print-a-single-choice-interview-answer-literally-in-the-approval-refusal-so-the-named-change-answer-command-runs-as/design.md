---
change: print-a-single-choice-interview-answer-literally-in-the-approval-refusal-so-the-named-change-answer-command-runs-as
artifact: design
---

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
