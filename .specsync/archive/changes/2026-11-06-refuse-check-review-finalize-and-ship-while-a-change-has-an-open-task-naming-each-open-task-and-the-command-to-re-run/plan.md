---
change: refuse-check-review-finalize-and-ship-while-a-change-has-an-open-task-naming-each-open-task-and-the-command-to-re-run
artifact: plan
---

# Plan

## Approach

Write the failing gate-level test first, under workflow v2, through the binary. Then factor the open-task parser and its sentence. Then give every refusing verb the same gate with its own re-run command, and teach status and ship-status the same sentence.

## Out of scope

- Workflow-v1 `change accept` and `change archive`. They are legacy verbs; `check` and `verify` already refuse there, and the new status prefix covers v1 too.
- The `Handoff:` line. It reports whether walking away is safe, which open tasks recorded on disk do not change.
- Putting checkbox state into any digest. Ticking a task must stay free (CHANGE-6.c).
- Changing what counts as a task: same parser, same fence handling.

## Steps

1. Failing integration test in `tests/integration/change.rs`. It runs the check refusal plus the named command; status text and JSON; reopen after a passing check; then review, finalize, ship and ship-status refusals, each followed by its named command.
2. Unit tests in `src/change_tests/open_task_gate.rs`: line numbers, fenced tasks ignored, the five-name bound, text sanitizing, and the verdict exemption for review.
3. `open_tasks` and `open_tasks_next_step` in `collect_requirement_ids.rs`; `ensure_tasks_complete` takes the re-run command.
4. `CheckInvocation` and `check_change_as` in `sdd_version.rs`; verify passes its own command.
5. Review and finalize gates, and the library `next_action` prefix, in `historical_definition_digest_matches.rs`.
6. CLI: `run_checked_commit` passes its flags; `run_ship` refuses; `ship_status_report` adds the blocker and prefixes `Next:`; `text_mode_next_action` prefixes.
7. Run the SS-M05 mutation (guard bypassed) and confirm the new test fails, then restore. Record it as evidence.
8. Specs (Public API rows, Change Log, version), the workflow docs recipe, CHANGELOG, then `fledge lanes run verify`.

## Risks

- A refusal at review could block recording a negative verdict; it is limited to `pass`.
- Ticking a box after review must not stale the review. The test runs exactly that path, from refusal through the named `ship` command to the archive.
- Two next-action sites could drift; both call one sentence builder, and the test asserts text and JSON.

## Constraints consulted

- specs/change/change.3md: reproduce workflow-v2 defects under workflow v2, and assert refusal text, not only the exit status.
- The `verification_ignored_paths` lesson: status and ship-status ask the gate directly instead of reading a refusal as stale.
- 95% plan rules: failing test first; a refusal names a command that actually recovers, and a test runs it; no test, gate, bound or timeout is weakened.
