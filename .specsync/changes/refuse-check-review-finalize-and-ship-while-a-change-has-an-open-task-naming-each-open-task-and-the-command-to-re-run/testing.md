---
change: refuse-check-review-finalize-and-ship-while-a-change-has-an-open-task-naming-each-open-task-and-the-command-to-re-run
artifact: testing
---

# Testing

## Plan

- `open_task_gate_refuses_and_names_the_command_that_recovers` (tests/integration/change.rs), workflow v2, through the binary:
  1. One open task. `change check <id> --commit` exits 1, writes no `verification.json`, and its stderr names `tasks.md`, the line, the text and `specsync change check <id> --commit`.
  2. Text and JSON status name the same task before `specsync change check <id>`.
  3. Tick the box and run the exact command the refusal named: it passes.
  4. Reopen the box after the passing check. Status names the task before review. `change review` exits 1, writes no `review.json`, and names `specsync change review <id> --reviewer <reviewer>`. Tick the box and run that command: it records the pass.
  5. Reopen the box again. `change ship-status` lists the blocker and is not ready. `change finalize` exits 1 with nothing archived and names `specsync change finalize <id>`. `change ship` exits 1 with nothing archived and names `specsync change ship <id>`.
  6. Tick the box and run the named ship command: it archives the change.
- Unit tests in `src/change_tests/open_task_gate.rs`: line numbers; fenced tasks ignored; the five-name bound and the remaining count; control-character and backtick sanitizing; a `fail` review verdict is still recorded with an open task.
- Mutation SS-M05 (the guard reports no open task): the integration test fails. The mutant receipt is recorded as Chough evidence; the source is restored and the test passes again.
- `fledge lanes run verify` passes. The existing task tests keep their assertions.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-131 | integration test open_task_gate_refuses_and_names_the_command_that_recovers (check, review and finalize legs); unit tests in src/change_tests/open_task_gate.rs; SS-M05 mutant fails the test |
| REQ-change-132 | integration test open_task_gate_refuses_and_names_the_command_that_recovers (text and JSON status before and after the passing check) |
| REQ-cmd-change-031 | integration test open_task_gate_refuses_and_names_the_command_that_recovers (ship-status blocker, ship refusal, named ship command archives) |
