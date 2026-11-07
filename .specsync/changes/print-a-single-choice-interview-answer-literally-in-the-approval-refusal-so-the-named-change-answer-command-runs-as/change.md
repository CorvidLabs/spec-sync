---
id: print-a-single-choice-interview-answer-literally-in-the-approval-refusal-so-the-named-change-answer-command-runs-as
state: implementing
type: bug_fix
base_commit: db6419678a19b5814e0b6c1ece58eb3543d8402e
---

# Print a single-choice interview answer literally in the approval refusal, so the named change answer command runs as printed (#958)

## Intent

Print a single-choice interview answer literally in the approval refusal, so the named change answer command runs as printed (#958)

## Affected Canonical Specs

- `change`
- `cmd_change`

## Acceptance Criteria

- The unfinished-interview refusal for a question with exactly one choice names that choice literally, e.g. `specsync change answer <id> constraints none`, so the command runs as printed (REQ-change-133, DOCS-3.a, proposed DOCS-3.b).
- A question with two or more choices keeps the `<a|b>` placeholder (`<yes|no>`), and one with no choices keeps `"<answer>"`.
- The documented-workflow probe runs the single-choice command exactly as printed: the harness no longer rewrites `<none>`.
- The approval guard refuses exactly the definitions it refused before; only the refusal text for a single-choice question changes.
- The changed unit test and the documented-workflow probe fail on main db641967 before the change and pass after it.

## No-spec Rationale

Not applicable
