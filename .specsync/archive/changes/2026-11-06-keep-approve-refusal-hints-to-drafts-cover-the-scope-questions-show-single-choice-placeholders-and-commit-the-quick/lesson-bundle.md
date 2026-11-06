# Lesson bundle: keep-approve-refusal-hints-to-drafts-cover-the-scope-questions-show-single-choice-placeholders-and-commit-the-quick

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Keep approve refusal hints to drafts, cover the scope questions, show single-choice placeholders, and commit the Quick Start source (#917)
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/collect_requirement_ids.rs, src/change_tests/, src/change_tests.rs, site/src/content/docs/index.md, tests/integration/documented_workflows.rs, specs/change/, CHANGELOG.md, .chough/
- **Acceptance**: A draft without an affected path is refused with `specsync change answer <id> affected_paths "<path>"` named, and a draft without an affected spec (and no justified no-spec change) with `specsync change answer <id> affected_specs "<module>"`; running them moves approval on (REQ-change-133, DOCS-3.a).
- **Acceptance**: An approved, implementing or verifying record that fails the same definition checks is refused with the same condition text and no `change answer` command, because that command refuses outside draft (REQ-change-133).
- **Acceptance**: The unfinished-interview refusal shows the first question choices as the placeholder whenever it has any (`<yes|no>`, `<none>` for constraints) and `"<answer>"` only when it has none, as REQ-change-133 says.
- **Acceptance**: The approval guard refuses exactly the definitions it refused before; only refusal text changes.
- **Acceptance**: The landing page Quick Start commits the files it created before the first change, so following it as written leaves a clean tree when the reader pushes; landing_page_change_workflow_runs_from_its_quick_start starts from exactly the documented state with no harness commit (DOCS-1).
- **Acceptance**: The new unit tests and the two changed integration tests fail at 2b30841d and pass after the change; the README and the other landing-page tests keep passing.

## Evidence

- Verification commit: `c60b4f47666db37759b99a2f27c786e54352dbbc`
- Base commit: `937508347cf321d2d45fd33fb3ca6916d08d87c7`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Issue #917 collects the four P2 findings from the Codex review of #912, the #883 change that added REQ-change-133. All four are real. #883's change is archived, and #912 is held for Leif's morning, so they are fixed here as one follow-up instead of on #912.

1. r4489583812: `interview_incomplete_refusal` gives `"<answer>"` for a single-choice question (`constraints`, choices `["none"]`). REQ-change-133 says the choices are the placeholder.
2. r4489583819: `validate_definition` also runs for approved, implementing and verifying records (check, verification, audit). A corrupted or hand-edited non-draft record is told to run `change answer`, which refuses outside draft. A refusal must name a command that can recover (the orc's standing rule for this work), and REQ-change-133 speaks of drafts.
3. r4489810025: a draft from `change new "..."` without `--path`/`--spec` gets the bare `at least one affected path is required` / `affected specs are required unless no_spec_change is justified`. Both are interview questions (`next_questions` asks them).
4. r4489810033: the landing page committed only `.specsync` after `change adopt`. Run from exactly the documented state, the reader pushes with `src/auth.ts`, `.gitignore` and `.gitattributes` (the last two from `specsync init`) uncommitted. #883's test hid this behind its own scaffold commit.

The branch leif/917-approve-hints is stacked on #912's archive tip 93750834. origin/main is merged in once #912 lands. The orc held #912 overnight (2026-11-06) and asked for #917 to be built to its approval gate, which queues for Leif.

## From the change's design.md

# Design

## `src/change/collect_requirement_ids.rs`

- `validate_definition` computes `draft = record.state == ChangeState::Draft` once. Each of the four refusals keeps its condition, order and leading text. Only for a draft, it appends `; run \`specsync change answer <id> <question> <placeholder>\``:
  - acceptance criterion: `acceptance_criteria "<observable outcome>"`
  - affected path: `affected_paths "<path>"`
  - affected spec: `affected_specs "<module>"`
  - interview: the first open question and its placeholder (below), after `(still to answer: ...)`
- Past draft, the text is exactly what 6.2.0 printed: `at least one acceptance criterion is required`, `at least one affected path is required`, `affected specs are required unless no_spec_change is justified`, `the deterministic interview is incomplete`.
- `interview_incomplete_refusal` uses `<choices joined by |>` whenever the question has any choice (`<yes|no>`, `<none>`), and `"<answer>"` only when it has none.

## Landing page (`site/src/content/docs/index.md`)

`git add .specsync && git commit -m "Adopt the SpecSync change workflow"` becomes `git add -A && git commit -m "Add SpecSync and adopt the change workflow"`. That commits what the Quick Start made: `.specsync/`, the `.gitignore` and `.gitattributes` from `init`, `specs/auth/` and `src/auth.ts`.

## Tests

- `src/change_tests/approve_refusal_hints.rs` (unit, `validate_definition` on an edited workflow-v2 record):
  - a draft without paths or without specs names the matching answer;
  - approved, implementing and verifying records with a missing criterion, path or interview answer never name `change answer` and keep the condition text;
  - single, paired and choice-less questions get `<none>`, `<yes|no>` and `"<answer>"`.
- `tests/integration/documented_workflows.rs`:
  - `landing_page_quick_start` no longer commits anything itself;
  - the DOCS-3.a probe runs the page's commands in order through `change new`;
  - the new `approve_refusals_for_a_draft_created_without_scope_name_the_scope_answers` runs the page up to `change new`, creates `specsync change new "Add auth" --json` without scope, and runs every named command (`"<path>"` becomes `src/auth.ts`, `"<module>"` becomes `auth`) until only the artifacts remain. It must have named the `acceptance_criteria`, `affected_paths` and `affected_specs` answers.

## From the change's testing.md

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

## Where these lessons go

- `specs/change/change.3md`
