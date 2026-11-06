---
id: refuse-check-review-finalize-and-ship-while-a-change-has-an-open-task-naming-each-open-task-and-the-command-to-re-run
state: implementing
type: bug_fix
base_commit: ad50bdef3db58a715877877d2407485bcaed0f32
---

# Refuse check, review, finalize and ship while a change has an open task, naming each open task and the command to re-run (#892)

## Intent

Refuse check, review, finalize and ship while a change has an open task, naming each open task and the command to re-run (#892)

## Affected Canonical Specs

- `change`
- `cmd_change`

## Acceptance Criteria

- change check, change review (pass verdict), change finalize and change ship each refuse a change with an open task in its tasks.md before writing any record, including a task reopened after a passing check, and each refusal names the task file, every open task by line and text, and the exact command to re-run.
- change status (text Next: and JSON summary.next_action) and change ship-status name the open tasks before the command that would otherwise be next, and ship-status reports them as a blocker.
- A gate-level test drives each refusal through the binary, ticks the box and runs the exact command the refusal named, which succeeds; with the incomplete-task guard bypassed (audit mutation SS-M05) that test fails.

## No-spec Rationale

Not applicable
