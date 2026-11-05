# Lesson bundle: stop-waiting-on-a-descendant-pipe-after-the-git-child-exits

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Stop waiting on a descendant pipe after the git child exits
- **Kind**: Feature
- **Specs**: git_utils, change, changelog, cmd_change, cmd_diff, cmd_lifecycle, comment, github, hooks, merge
- **Paths**: src/git_utils.rs, src/change/consider_accepted_evidence_anchor.rs, src/change/collect_requirement_ids.rs, src/change/historical_definition_digest_matches.rs, src/change/validate_loaded_change.rs, src/change_tests/substantive_plan.rs, src/changelog.rs, src/commands/change.rs, src/commands/diff.rs, src/commands/lifecycle.rs, src/comment.rs, src/github.rs, src/hooks.rs, src/merge.rs, hi/speed.md, specs/git_utils/git_utils.spec.md, specs/git_utils/git_utils.3md, specs/git_utils/requirements.md, specs/git_utils/tasks.md, specs/git_utils/testing.md, specs/change/change.spec.md, specs/change/change.3md, specs/change/requirements.md, specs/change/tasks.md, specs/change/testing.md
- **Acceptance**: A Git read returns the child's own bytes while a descendant still holds that child's output pipe. A status Git child does not receive the caller's stdout. The existing deadline, output cap, and stdin-writer join checks still pass. SPEED-1, SPEED-1.a, and SPEED-1.e.

## Evidence

- Verification commit: `ffa975fffa915924e1730a1a73ed08518f218c9c`
- Base commit: `40dcf33a6b0f0b55dcb7f17e8e98be9d55fa2c56`
- Verified by: `specsync check --spec change --spec changelog --spec cmd_change --spec cmd_diff --spec cmd_lifecycle --spec comment --spec git_utils --spec github --spec hooks --spec merge`

## From the change's context.md

# Context

`std::process::Command::output` and the bounded Git runner both read stdout until end-of-file. End-of-file arrives only when every process that inherited the pipe has closed it. A grandchild can keep that pipe after Git itself has exited: a hook, a credential helper, `git gc --auto`, or a wrapper that backgrounds a process and then execs Git.

That is #855: `specsync change status` does not return. Chough bounds its read of SpecSync at 120 seconds, so `chough status` reports timed-out (#854). Both issues are this one wait.

The fix does not add a timeout and does not kill the grandchild. It stops reading once the direct child has been reaped, and it does not hand the caller's stdout or stderr to a status child. The existing deadline, cap, and stdin-writer join stay in place.

Leif's bar, captured as SPEED-1.e: SpecSync must never wait on a descendant's open pipe after its child exits.

## From the change's design.md

# Design

No user-interface change. The reader threads poll the child's pipe and a wake pipe. When the direct child is reaped, one byte on the wake pipe releases every reader. They keep bytes already buffered and do not wait for the write end to close.

## From the change's testing.md

# Testing

`bounded_git_read_returns_while_a_descendant_holds_the_output_pipe` and `output_returns_while_a_descendant_holds_the_pipe` run a child that writes `payload` and exits while a grandchild blocks on a fifo holding stdout. The assertion is those bytes and that the grandchild is still alive. Before the fix the bounded test waited the full receive bound with the holder alive. The receive bound only reports that the read never returned. It is not the success condition.

`status_does_not_hand_the_caller_stdout_to_the_child` records the path of the child's stdout and requires the null device.

`stop_keeps_queued_bytes_and_does_not_follow_a_descendant_flood` fills the pipe, blocks a descendant in `write`, then reads with stop already set. The payload at the head is kept and the length does not exceed the pipe capacity. A receive bound only reports that the flood was followed.

`output_after_child_exits_nulls_stdin` points a command whose stdin was a pipe at the null device.

These stay as they were, and were re-run after the fix: `bounded_git_runner_times_out_a_silent_process`, `timed_out_git_runner_reaps_child_and_joins_blocked_writer`, `timed_out_git_runner_reaps_child_before_child_can_make_progress`.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-126 | `bounded_git_read_returns_while_a_descendant_holds_the_output_pipe`, `bounded_git_runner_times_out_a_silent_process`, `timed_out_git_runner_reaps_child_and_joins_blocked_writer`, `timed_out_git_runner_reaps_child_before_child_can_make_progress`, `capped_git_runner_drains_kills_and_reaps_large_output` |
| REQ-git-utils-008 | `output_returns_while_a_descendant_holds_the_pipe`, `status_does_not_hand_the_caller_stdout_to_the_child`, `output_after_child_exits_nulls_stdin`, `stop_keeps_queued_bytes_and_does_not_follow_a_descendant_flood` |

## Where these lessons go

- `specs/git_utils/git_utils.3md`
- `specs/change/change.3md`
- `specs/changelog/changelog.3md`
- `specs/cmd_change/cmd_change.3md`
- `specs/cmd_diff/cmd_diff.3md`
- `specs/cmd_lifecycle/cmd_lifecycle.3md`
- `specs/comment/comment.3md`
- `specs/github/github.3md`
- `specs/hooks/hooks.3md`
- `specs/merge/merge.3md`
