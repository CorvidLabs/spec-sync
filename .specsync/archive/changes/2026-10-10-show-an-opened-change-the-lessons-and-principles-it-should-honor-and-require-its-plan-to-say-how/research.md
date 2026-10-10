---
change: show-an-opened-change-the-lessons-and-principles-it-should-honor-and-require-its-plan-to-say-how
artifact: research
---

# Research

Finding 10 in `docs/6-0-findings.md` says archival is where knowledge moves into `specs/<module>/context.md`, and that `finalize` should assemble a bundle and point at it. The bundle writer and `accumulated_lessons` landed. The pointer was then removed from `next_action` because it was read as a merge gate. `ship_does_not_gate_merge_on_writing_lessons` locks that. JSON still carries `lesson_bundle`. Text does not.

`print_accumulated_lessons` runs only from the human-text arm of `change new`. Agents follow `--json`.

`principles_file` is part of the execution-digest snapshot and must be non-empty at approval. Nothing in the interview or in `specsync rules` reads it aloud. The configuration page claims it enters the interview.

`adaptive_artifacts` gives a plan to feature, refactor, migration, and operations changes. A bug fix gets testing and tasks. A documentation change gets docs. More than one spec or more than four paths adds design and tasks. Answers can add artifacts and cannot remove them.

The scope digest serializes `ApprovedScopeV1`, whose `answers` map already holds `public_contract` and `architecture_risk`. Adding a key is digest-safe for records that do not have the key.
