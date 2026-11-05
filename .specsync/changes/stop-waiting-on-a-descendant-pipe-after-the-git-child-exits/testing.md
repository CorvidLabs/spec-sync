---
change: stop-waiting-on-a-descendant-pipe-after-the-git-child-exits
artifact: testing
---

# Testing

`bounded_git_read_returns_while_a_descendant_holds_the_output_pipe` and `output_returns_while_a_descendant_holds_the_pipe` run a child that writes `payload` and exits while a grandchild blocks on a fifo holding stdout. The assertion is those bytes and that the grandchild is still alive. Before the fix the bounded test waited the full receive bound with the holder alive. The receive bound only reports that the read never returned. It is not the success condition.

`status_does_not_hand_the_caller_stdout_to_the_child` records the path of the child's stdout and requires the null device.

These stay as they were, and were re-run after the fix: `bounded_git_runner_times_out_a_silent_process`, `timed_out_git_runner_reaps_child_and_joins_blocked_writer`, `timed_out_git_runner_reaps_child_before_child_can_make_progress`.
