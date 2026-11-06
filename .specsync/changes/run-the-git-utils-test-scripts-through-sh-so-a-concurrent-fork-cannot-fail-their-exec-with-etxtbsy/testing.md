---
change: run-the-git-utils-test-scripts-through-sh-so-a-concurrent-fork-cannot-fail-their-exec-with-etxtbsy
artifact: testing
---

# Testing

Failing first: on main 884a33d3, in a `rust:1.89` container, `specsync-<hash> git_utils::tests --test-threads=16` run 300 times failed 9 rounds with ETXTBSY (`status_does_not_hand_the_caller_stdout_to_the_child` 7, `output_returns_while_a_descendant_holds_the_pipe` 2).

With the change: the same 300-round run must fail 0 rounds, and the two tests keep their assertions (the child's stdout is the null device; the read returns the child's bytes while the descendant is alive).
