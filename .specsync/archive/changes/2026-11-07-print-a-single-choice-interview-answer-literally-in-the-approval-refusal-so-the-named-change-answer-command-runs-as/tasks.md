---
change: print-a-single-choice-interview-answer-literally-in-the-approval-refusal-so-the-named-change-answer-command-runs-as
artifact: tasks
---

# Tasks

- [x] Capture DOCS-3.b and DOCS-3.c as approved, with their source, in `hi/docs.md`.
- [x] Write the failing tests and see them fail on main db641967.
- [x] `answer_command_hint`, the specs-before-paths order, and both status next actions.
- [x] specs/change and specs/cmd_change: materialized requirements, Error Cases row, version bumps, Change Log rows, testing.md lines, a cmd_change.3md lesson plane; CHANGELOG Unreleased entry.
- [x] Run the five hint tests and both documented-workflow probes.

Verification gates, not tasks: run `fledge lanes run verify` and `specsync check --strict`, then record the product evidence.
Human gates, not tasks: Leif's scope approval of this change, and his implementation review before `change review` and `ship` (both relayed by orc t18).
