---
change: refuse-check-review-finalize-and-ship-while-a-change-has-an-open-task-naming-each-open-task-and-the-command-to-re-run
artifact: design
---

# Design

- One parser for open tasks: `markdown_task_checkbox_offsets` filtered to `[ ]`, with the 1-based line and the task text. Fenced code stays excluded, exactly as today.
  - The text is trimmed, control characters are dropped, backticks become `'`, and it is capped at 80 characters.
  - At most five tasks are named; the rest are counted ("and 2 more").
- One sentence for the fix, shared by every surface (`change::open_tasks_next_step`): "finish the 1 open task in `.specsync/changes/<id>/tasks.md` (line 12: "Second task.") and mark it `[x]`". The plural form says "mark each one `[x]`".
- Refusal: `tasks.md contains incomplete tasks: <next step>, then re-run `<command>``. The old prefix is kept, so anything matching it still matches.
- The command is the one that refused, with its flags:
  - check: `specsync change check <id>`, plus `--strict`, `--commit` and `--push` as given. This uses a new `CheckInvocation` passed by the CLI; `check_change_with_strict` stays as a wrapper.
  - verify: `specsync change verify <id>` (`--strict`).
  - review: `specsync change review <id> --reviewer <reviewer>`, single-quoted if it contains a space.
  - finalize: `specsync change finalize <id>`.
  - ship: `specsync change ship <id>` with `--dry-run`, `--push` and `--wait` as given.
- Where the gate runs, each time before any record is written:
  - review: after state and approval checks, for a `pass` verdict only. A non-pass verdict on unfinished work stays legitimate.
  - finalize: after the legacy and materialization checks, before acceptance, for a Verifying change. A resumed finalization (Accepted) already passed the gate when it was accepted, and its workspace may already be in the archive. `post_move_same_pr_finalization_resumes_across_dates_without_another_approval` caught the first cut, which also gated the resume.
  - ship: right before its readiness refusal, so the message is not doubled.
  - ship-status: an incomplete-tasks blocker, so `ready_to_finalize` is false.
- Status: for Approved, Implementing and Verifying, with a current approval and complete artifacts, `Next:` becomes `<next step>, then <what it would otherwise say>`. This holds in the CLI text and in the library JSON `next_action`. Sequence-freeze and `verification_ignored_paths` actions still come first.
- No digest, record schema or persisted field changes. Checkbox state stays out of the approval and verification digests, so ticking a task still needs no reapproval and no re-verification.
