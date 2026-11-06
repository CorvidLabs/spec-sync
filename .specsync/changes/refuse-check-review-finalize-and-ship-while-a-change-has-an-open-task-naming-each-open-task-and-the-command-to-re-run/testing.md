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
- Unit tests in `src/change_tests/open_task_gate.rs`: line numbers; fenced tasks ignored; the five-name bound and the remaining count; control-character and backtick sanitizing; check-command flags; reviewer quoting. The `block` review verdict exemption runs end to end in the integration test.
- Mutation SS-M05 (the guard reports no open task): the integration test fails. The mutant receipt is recorded as Chough evidence; the source is restored and the test passes again.
- `fledge lanes run verify` passes. The existing task tests keep their assertions.

## Results (local macOS, debug build, rustc 1.99.0)

- Red first, on unmodified code plus the new test: the test failed at the first refusal. The whole stderr was `error: tasks.md contains incomplete tasks`.
- Green at 20cad7f5: the gate-level test passes in 160s. The six unit tests and the two `ship_command_tests` pass.
- The block-verdict exemption is covered end to end. A pass review is refused with the task open; `--verdict block` is recorded; ticking the box and running the named pass command records `pass`.
- Mutations at 20cad7f5: one at a time, each source restored byte for byte. Every mutant fails the gate-level test:

| Mutant | Change | Where the test fails |
| --- | --- | --- |
| SS-M05 (audit equivalent) | parser reports no open task | first `change check --commit`: unexpected success |
| M892-guard | `ensure_tasks_complete` always passes | first `change check --commit`: unexpected success |
| M892-review | a pass review is not gated | `change review`: unexpected success |
| M892-finalize | finalize is not gated | `change finalize`: unexpected success |
| M892-ship | ship has no refusal of its own | ship refusal names no command to re-run |
| M892-status-json | library `next_action` has no prefix | JSON status must name the open task first |
| M892-status-text | text `Next:` has no prefix | text status must name the open task first |
| M892-ship-status | no ship-status blocker | `ready_to_finalize` / blocker assertion |
| M892-finalize-resume | an Accepted retry with an active workspace is not gated | `a_resumed_finalization_still_in_its_workspace_refuses_a_reopened_task`: the retry archived |
| M892-reviewer-hyphen | the named review command uses `--reviewer <claim>` | `the_named_review_command_parses_for_every_valid_reviewer`: `--reviewer -bot` does not parse |

The whole matrix (all ten) was re-run at 1d545478, after the Codex fixes: 10 of 10 killed, each source restored byte for byte.

- The first full `cargo test` run (pinned 1.89) failed one test: `post_move_same_pr_finalization_resumes_across_dates_without_another_approval`. The finalize gate also ran on a resumed, already-moved finalization and could not find the active `tasks.md`. At 261bd4aa the gate runs only before acceptance (Verifying). That test, the 22 `finaliz` unit tests and the gate-level test pass, and M892-finalize, re-run against the moved gate, is still killed ("unexpected success" at finalize).

- Codex review of #903 (two P2s, both fixed at 1d545478, each with a test and a mutant):
  - A finalization retried after acceptance, with its workspace still active, now refuses a reopened task. Test: `a_resumed_finalization_still_in_its_workspace_refuses_a_reopened_task`.
  - The named review command attaches the reviewer with `=`. Test: `the_named_review_command_parses_for_every_valid_reviewer`, which parses `-bot`, a name with a space and `ci:bot/@x-1` through the real CLI.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-131 | integration test open_task_gate_refuses_and_names_the_command_that_recovers (check, review and finalize legs); unit tests in src/change_tests/open_task_gate.rs, including a_resumed_finalization_still_in_its_workspace_refuses_a_reopened_task and the_named_review_command_parses_for_every_valid_reviewer; SS-M05 and per-gate mutants fail them |
| REQ-change-132 | integration test open_task_gate_refuses_and_names_the_command_that_recovers (text and JSON status before and after the passing check) |
| REQ-cmd-change-031 | integration test open_task_gate_refuses_and_names_the_command_that_recovers (ship-status blocker, ship refusal, named ship command archives) |
