# Lesson bundle: refuse-check-review-finalize-and-ship-while-a-change-has-an-open-task-naming-each-open-task-and-the-command-to-re-run

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Refuse check, review, finalize and ship while a change has an open task, naming each open task and the command to re-run (#892)
- **Kind**: BugFix
- **Specs**: change, cmd_change
- **Paths**: src/change/collect_requirement_ids.rs, src/change/sdd_version.rs, src/change/historical_definition_digest_matches.rs, src/commands/change.rs, src/change_tests.rs, src/change_tests/open_task_gate.rs, tests/integration/change.rs, site/src/content/docs/workflow.md, CHANGELOG.md, specs/change/, specs/cmd_change/, hi/change.md, INTENT.md, .chough/
- **Acceptance**: change check, change review (pass verdict), change finalize and change ship each refuse a change with an open task in its tasks.md before writing any record, including a task reopened after a passing check, and each refusal names the task file, every open task by line and text, and the exact command to re-run.
- **Acceptance**: change status (text Next: and JSON summary.next_action) and change ship-status name the open tasks before the command that would otherwise be next, and ship-status reports them as a blocker.
- **Acceptance**: A gate-level test drives each refusal through the binary, ticks the box and runs the exact command the refusal named, which succeeds; with the incomplete-task guard bypassed (audit mutation SS-M05) that test fails.

## Evidence

- Verification commit: `30025a3cc8ba42830e2119799f5a163d923bf1b0`
- Base commit: `ad50bdef3db58a715877877d2407485bcaed0f32`
- Verified by: `specsync check --spec change --spec cmd_change`

## From the change's context.md

# Context

- Audit SS-M05 (2026-11-05): bypassing the incomplete-task guard left every selected test passing. The follow-up probe found the refusal gives no recovery and status repeats the check command. The 95% plan row S6 asks for a mutation-checked gate-level test and a named recovery.
- While scoping, a fixture showed a worse gap: a task reopened after a passing check was reviewed and finalized. Leif chose to widen this change to cover it ("Widen S6", 2026-11-05 20:24 MST, relayed by the orc t18). See #892.
- Rejected: hashing checkbox state into a digest. That would make ticking a task cost re-verification, or reapproval, contrary to CHANGE-6.c. A direct gate at each verb is enough.
- Human gates: scope approval and implementation review come from Leif through the orc; this agent never records them as Leif.

## From the change's design.md

# Design

- One parser for open tasks: `markdown_task_checkbox_offsets` filtered to `[ ]`, with the 1-based line and the task text. Fenced code stays excluded, exactly as today.
  - The text is trimmed, control characters are dropped, backticks become `'`, and it is capped at 80 characters.
  - At most five tasks are named; the rest are counted ("and 2 more").
- One sentence for the fix, shared by every surface (`change::open_tasks_next_step`): "finish the 1 open task in `.specsync/changes/<id>/tasks.md` (line 12: "Second task.") and mark it `[x]`". The plural form says "mark each one `[x]`".
- Refusal: `tasks.md contains incomplete tasks: <next step>, then re-run `<command>``. The old prefix is kept, so anything matching it still matches.
- The command is the one that refused, with its flags:
  - check: `specsync change check <id>`, plus `--strict`, `--commit` and `--push` as given. This uses a new `CheckInvocation` passed by the CLI; `check_change_with_strict` stays as a wrapper.
  - verify: `specsync change verify <id>` (`--strict`).
  - review: `specsync change review <id> --reviewer=<reviewer>`, attached with `=` so a claim starting with `-` parses, and single-quoted if it contains a space (Codex review of #903).
  - finalize: `specsync change finalize <id>`.
  - ship: `specsync change ship <id>` with `--dry-run`, `--push` and `--wait` as given.
- Where the gate runs, each time before any record is written:
  - review: after state and approval checks, for a `pass` verdict only. A non-pass verdict on unfinished work stays legitimate.
  - finalize: after the legacy and materialization checks, before acceptance (Verifying), and again when a finalization that stopped after acceptance is retried while its workspace is still under `.specsync/changes/`, because a task can be reopened in between. A workspace already moved into the archive resumes; it passed the gate when it was accepted. The first cut gated every resume, and `post_move_same_pr_finalization_resumes_across_dates_without_another_approval` caught it. The second cut skipped every resume, and the Codex review of #903 caught that.
  - ship: right before its readiness refusal, so the message is not doubled.
  - ship-status: an incomplete-tasks blocker, so `ready_to_finalize` is false.
- Status: for Approved, Implementing and Verifying, with a current approval and complete artifacts, `Next:` becomes `<next step>, then <what it would otherwise say>`. This holds in the CLI text and in the library JSON `next_action`. Sequence-freeze and `verification_ignored_paths` actions still come first.
- No digest, record schema or persisted field changes. Checkbox state stays out of the approval and verification digests, so ticking a task still needs no reapproval and no re-verification.

## From the change's testing.md

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

## Where these lessons go

- `specs/change/change.3md`
- `specs/cmd_change/cmd_change.3md`
