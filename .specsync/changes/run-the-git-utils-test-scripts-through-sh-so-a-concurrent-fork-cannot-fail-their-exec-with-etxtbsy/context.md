---
change: run-the-git-utils-test-scripts-through-sh-so-a-concurrent-fork-cannot-fail-their-exec-with-etxtbsy
artifact: context
---

# Context

PR #868's CI run failed once in `git_utils::tests::status_does_not_hand_the_caller_stdout_to_the_child` with `Os { code: 26, kind: ExecutableFileBusy, message: "Text file busy" }`. Main passed the same test, and a rerun of only that job passed.

Two git_utils tests write a shell script, mark it executable and run it directly: `status_does_not_hand_the_caller_stdout_to_the_child` and `output_returns_while_a_descendant_holds_the_pipe`. If another test thread forks while the script's write descriptor is still open, the forked child holds that descriptor until it execs, and Linux refuses to exec a file open for writing with ETXTBSY.

On main 884a33d3, a stress run in a `rust:1.89` Linux container (`git_utils::tests --test-threads=16`, 300 rounds) failed 9 rounds: 7 in the status test and 2 in the descendant-pipe test, all ETXTBSY.

Running the script as `sh <path>` means the kernel execs `/bin/sh`, never the freshly written file, so the race cannot occur. The script body and every assertion stay the same.

Out of scope (same pattern, not seen failing, other modules): `src/change_tests/semantic_application_resolves_registry_backed_canonical_paths.rs:3093`, `src/change_tests/substantive_plan.rs:3405` and `:3492`, `src/github.rs:2357` and `:2479`, `tests/integration/commands/issues.rs:1075`.
