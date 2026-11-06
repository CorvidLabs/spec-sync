## ADDED

### REQUIREMENT REQ-change-131

A lifecycle command that verifies a change, records a passing scoped review of it, or finalizes it SHALL refuse while the change's selected `tasks.md` has an open task, a `[ ]` checkbox outside fenced code. The refusal SHALL come before the command writes any record. This SHALL hold for a task reopened after a passing check, although checkbox state is outside the approval and verification digests. The refusal SHALL name:

- the task file;
- each open task by line and text, naming at most five and counting the rest;
- the exact command to re-run once they are done.

`change check` SHALL name `specsync change check <id>` with the `--strict`, `--commit` and `--push` flags it was given. `change verify` SHALL name `specsync change verify <id>`. `change review` SHALL name `specsync change review <id> --reviewer <reviewer>`. `change finalize` SHALL name `specsync change finalize <id>`. A scoped review recorded with a verdict other than `pass` SHALL NOT be refused.

#### Acceptance Criteria

- Through the binary, `change check <id> --commit` on an approved workflow-v2 change with one open task exits 1 without recording verification. Its stderr names `tasks.md`, the task's line and text, and `specsync change check <id> --commit`. Ticking the box and running that exact command passes.
- After a passing check, reopening one task makes `change review` and `change finalize` exit 1 without writing a review, acceptance or archive. Each names the open task and its own command. Ticking the box and running the named command succeeds.
- The gate-level test fails when the incomplete-task guard is bypassed (audit mutation SS-M05).

### REQUIREMENT REQ-change-132

`change status` SHALL name the open tasks and how to close them before the command that would otherwise be next, instead of naming only that command. This SHALL apply to an approved, implementing or verifying change whose approval is current, whose artifacts are complete and whose `tasks.md` has an open task. It SHALL apply in the text `Next:` line and in JSON `summary.next_action`.

#### Acceptance Criteria

- With one open task on an approved change, text and JSON status name the task file, the task's line and text, and then `specsync change check <id>`.
- After a passing check with one task reopened, text and JSON status name the open task before the review step.
