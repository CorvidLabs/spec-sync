## ADDED

### REQUIREMENT REQ-change-126

A bounded Git read SHALL return when the direct child exits and SHALL NOT wait on a descendant's open pipe after that child exits. `change status` is one such read (SPEED-1, SPEED-1.a, SPEED-1.e). The wall-clock deadline, the output cap, and the stdin-writer join SHALL stay as they are.

#### Acceptance Criteria

- `bounded_git_read_returns_while_a_descendant_holds_the_output_pipe` returns the child's stdout while the descendant is still alive.
- `bounded_git_runner_times_out_a_silent_process`, `timed_out_git_runner_reaps_child_and_joins_blocked_writer`, `timed_out_git_runner_reaps_child_before_child_can_make_progress`, and `capped_git_runner_drains_kills_and_reaps_large_output` still pass. No deadline, cap, or join assertion is loosened.
