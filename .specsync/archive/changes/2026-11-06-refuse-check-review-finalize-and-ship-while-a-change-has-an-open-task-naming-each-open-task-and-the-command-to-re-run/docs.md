---
change: refuse-check-review-finalize-and-ship-while-a-change-has-an-open-task-naming-each-open-task-and-the-command-to-re-run
artifact: docs
---

# Docs

- `site/src/content/docs/workflow.md`: a short "When tasks are still open" recipe. Every verb refuses until the box is ticked, and the refusal names the command to re-run.
- `CHANGELOG.md` Unreleased: open tasks now stop review, finalize and ship too, and every refusal and status names the tasks and the command.
- Spec Change Log rows and Public API rows (`CheckInvocation`, `check_change_as`, `open_tasks_next_step`) for change; a Change Log row for cmd_change.
