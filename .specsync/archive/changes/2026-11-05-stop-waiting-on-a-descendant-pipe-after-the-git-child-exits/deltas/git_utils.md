## ADDED

### REQUIREMENT REQ-git-utils-008

`output_after_child_exits` SHALL return when the direct child exits and SHALL NOT wait on a descendant's open pipe after that child exits. `status_without_inherited_output` SHALL NOT give the child the caller's stdout or stderr.

Acceptance Criteria
- `output_returns_while_a_descendant_holds_the_pipe` returns the child's bytes while the descendant is still alive.
- `status_does_not_hand_the_caller_stdout_to_the_child` records the null device as the child's stdout.
- `output_after_child_exits_nulls_stdin` records the null device even when the command's stdin was a pipe.
- `stop_keeps_queued_bytes_and_does_not_follow_a_descendant_flood` returns the payload that was at the head of a full pipe while a descendant is blocked in `write`, and the returned length does not exceed the pipe capacity.
- SPEED-1 and SPEED-1.e. The deadline and output-cap behavior of the bounded Git runner is unchanged.
