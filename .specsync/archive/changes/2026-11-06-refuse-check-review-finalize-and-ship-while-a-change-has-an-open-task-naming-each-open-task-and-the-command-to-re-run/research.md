---
change: refuse-check-review-finalize-and-ship-while-a-change-has-an-open-task-naming-each-open-task-and-the-command-to-re-run
artifact: research
---

# Research

- Fixture probe on main ad50bdef (debug build), workflow v2, one task file with two ticked tasks:
  - One box open before check: `change check --commit` exits 1, and its whole stderr is `error: tasks.md contains incomplete tasks`. `change status` still says `Next: run specsync change check <id>`.
  - Box reopened after a passing check: status reports `Handoff: safe — verification is current`, `change review` records a pass, and `change finalize` archives the change with `- [ ] Second task.` in the archived `tasks.md`.
- Why the reopened task is invisible:
  - `canonical_definition_artifact_payload` blanks every checkbox before the definition digest, on purpose, so ticking a task needs no reapproval (CHANGE-6.c).
  - `project_input_is_volatile` leaves out all of `.specsync/changes/`.
  - So no currency digest sees checkbox state. Only `materialize_change_deltas` and `verify_change_locked` call `ensure_tasks_complete`.
- The audit's SS-M05 mutation (`.any(|offset| content[offset] == b' ')` becomes `.any(|_offset| false)`) left every selected test passing. No test asserts this refusal at the lifecycle level.
- Existing patterns to reuse:
  - `verification_ignored_paths_refusal` is asked directly by status and ship-status, so a refusal is never reported as stale.
  - `text_mode_next_action` (CLI text) and the library `next_action` (JSON) are two separate next-action sites, and both need the same prefix.
  - `change_ready_to_verify` in `tests/integration/change.rs` builds the fixture, but under workflow v1, so the new test needs a v2 policy.
