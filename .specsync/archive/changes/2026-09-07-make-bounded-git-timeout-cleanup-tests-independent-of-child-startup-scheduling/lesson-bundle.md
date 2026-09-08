# Lesson bundle — make-bounded-git-timeout-cleanup-tests-independent-of-child-startup-scheduling

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Make bounded Git timeout cleanup tests independent of child startup scheduling
- **Kind**: Feature
- **Specs**: change
- **Paths**: src/change.rs, src/change_tests.rs, specs/change
- **Acceptance**: Timeout cleanup regression obtains the spawned child PID in the parent without a child-written readiness file; it still asserts deadline failure and child termination/reaping with a blocked stdin payload; deterministic delayed-start and cleanup negative controls distinguish the race from cleanup defects; repeated targeted runs and required verification pass without changing production deadlines or release gates.

## Evidence

- Verification commit: `32bb49413f3e33b559d357f8195bdb2baa1757da`
- Base commit: `4908a238a0030a451b3647c2a708403704545157`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Issue #587 reproduces in rc.15 macOS qualification run 34164714867: the 50ms deadline kills the child before it writes child.pid. The PID-file read then fails despite successful runner cleanup. Current main is 4908a238; no open PR overlaps this fix. Scope is the bounded Git runner test seam, its regression tests, and the change module spec companions.

## From the change's testing.md

# Testing

Target cargo test timed_out_git_runner_reaps_child_and_joins_blocked_writer and bounded_git_runner tests. Reproduce the old missing-PID failure by delaying the child before its file write; the replacement must pass the equivalent delayed-start scenario. Temporarily disable the relevant cleanup operation in a controlled test copy and require failure with bounded execution and explicit cleanup afterward. Repeat the fixed focused test under parallel load. Run scoped change check, fledge lanes run verify, pre-push, fledge trust verify, and signed Attest verification. Hosted macOS qualification remains required before declaring release readiness.

## Requirement evidence

| ID | Evidence |
|----|----------|
| REQ-change-096 | `timed_out_git_runner_reaps_child_and_joins_blocked_writer`, `timed_out_git_runner_reaps_child_before_child_can_make_progress`: both pass; 100 repeated paired runs at concurrency 8 pass. Removing timeout reaping fails both at the reaping assertion; removing writer join fails both at the join assertion. Original child-written PID test with delayed startup fails with NotFound. |

## Where these lessons go

- `specs/change/context.md`
