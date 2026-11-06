---
id: keep-approve-refusal-hints-to-drafts-cover-the-scope-questions-show-single-choice-placeholders-and-commit-the-quick
state: implementing
type: bug_fix
base_commit: 937508347cf321d2d45fd33fb3ca6916d08d87c7
---

# Keep approve refusal hints to drafts, cover the scope questions, show single-choice placeholders, and commit the Quick Start source (#917)

## Intent

Keep approve refusal hints to drafts, cover the scope questions, show single-choice placeholders, and commit the Quick Start source (#917)

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- A draft without an affected path is refused with `specsync change answer <id> affected_paths "<path>"` named, and a draft without an affected spec (and no justified no-spec change) with `specsync change answer <id> affected_specs "<module>"`; running them moves approval on (REQ-change-133, DOCS-3.a).
- An approved, implementing or verifying record that fails the same definition checks is refused with the same condition text and no `change answer` command, because that command refuses outside draft (REQ-change-133).
- The unfinished-interview refusal shows the first question choices as the placeholder whenever it has any (`<yes|no>`, `<none>` for constraints) and `"<answer>"` only when it has none, as REQ-change-133 says.
- The approval guard refuses exactly the definitions it refused before; only refusal text changes.
- The landing page Quick Start commits the files it created before the first change, so following it as written leaves a clean tree when the reader pushes; landing_page_change_workflow_runs_from_its_quick_start starts from exactly the documented state with no harness commit (DOCS-1).
- The new unit tests and the two changed integration tests fail at 2b30841d and pass after the change; the README and the other landing-page tests keep passing.

## No-spec Rationale

Not applicable
