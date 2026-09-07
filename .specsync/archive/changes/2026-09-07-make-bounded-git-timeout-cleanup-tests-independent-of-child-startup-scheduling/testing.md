---
change: make-bounded-git-timeout-cleanup-tests-independent-of-child-startup-scheduling
artifact: testing
---

# Testing

Target cargo test timed_out_git_runner_reaps_child_and_joins_blocked_writer and bounded_git_runner tests. Reproduce the old missing-PID failure by delaying the child before its file write; the replacement must pass the equivalent delayed-start scenario. Temporarily disable the relevant cleanup operation in a controlled test copy and require failure with bounded execution and explicit cleanup afterward. Repeat the fixed focused test under parallel load. Run scoped change check, fledge lanes run verify, pre-push, fledge trust verify, and signed Attest verification. Hosted macOS qualification remains required before declaring release readiness.

## Requirement evidence

| ID | Evidence |
|----|----------|
| REQ-change-096 | `timed_out_git_runner_reaps_child_and_joins_blocked_writer`, `timed_out_git_runner_reaps_child_before_child_can_make_progress`: both pass; 100 repeated paired runs at concurrency 8 pass. Removing timeout reaping fails both at the reaping assertion; removing writer join fails both at the join assertion. Original child-written PID test with delayed startup fails with NotFound. |
