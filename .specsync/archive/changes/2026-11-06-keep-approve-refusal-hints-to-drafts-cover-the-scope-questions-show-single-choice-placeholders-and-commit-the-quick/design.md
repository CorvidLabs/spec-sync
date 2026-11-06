---
change: keep-approve-refusal-hints-to-drafts-cover-the-scope-questions-show-single-choice-placeholders-and-commit-the-quick
artifact: design
---

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
