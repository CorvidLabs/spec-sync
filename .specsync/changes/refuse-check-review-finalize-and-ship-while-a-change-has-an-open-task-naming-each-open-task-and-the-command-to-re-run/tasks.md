---
change: refuse-check-review-finalize-and-ship-while-a-change-has-an-open-task-naming-each-open-task-and-the-command-to-re-run
artifact: tasks
---

# Tasks

- [ ] Failing gate-level integration test, workflow v2, through the binary (check, status, review, finalize, ship, ship-status; each recovery command run)
- [ ] Unit tests for the open-task parser, bounds, sanitizing and the review verdict exemption
- [ ] Open-task parser and next-step sentence; `ensure_tasks_complete` names the re-run command
- [ ] `CheckInvocation` / `check_change_as`; verify names its own command
- [ ] Review (pass verdict) and finalize gates; library `next_action` prefix
- [ ] CLI: check --commit flags, ship refusal, ship-status blocker and Next, text status Next
- [ ] SS-M05 mutation run: guard bypassed, the new test fails; restored
- [ ] Specs, workflow docs recipe, CHANGELOG; `fledge lanes run verify`

Human gates (plain text, not tasks): scope approval and implementation review are relayed by the orc (t18) from Leif.
