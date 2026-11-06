---
change: refuse-check-review-finalize-and-ship-while-a-change-has-an-open-task-naming-each-open-task-and-the-command-to-re-run
artifact: requirements
---

# Requirements

- REQ-change-131 (added): check, verify, a passing scoped review and finalize refuse a change with an open task, including a task reopened after a passing check. The refusal names the task file, each open task by line and text, and the exact command to re-run.
- REQ-change-132 (added): `change status` names the open tasks before the command that would otherwise be next, in text and JSON.
- REQ-cmd-change-031 (added): `change ship` (including `--dry-run`) refuses an open task and names `specsync change ship <id>` with its flags; `change ship-status` reports a blocker, is not ready to finalize, and its `Next:` names the open tasks first.
- Traces to HI CHANGE-14, CHANGE-14.a, CHANGE-14.b and CHANGE-14.c (approved by Leif on 2026-11-05, relayed by the orc); issue #892; audit mutation SS-M05.
- IDs 129 and 130 are skipped on purpose: a sibling branch (#879) already adds REQ-change-128, and parallel S-row changes may add more. A gap avoids a duplicate-ID refusal when they merge.
