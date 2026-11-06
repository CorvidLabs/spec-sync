---
change: run-the-git-utils-test-scripts-through-sh-so-a-concurrent-fork-cannot-fail-their-exec-with-etxtbsy
artifact: context
---

# Context

PR #868's CI run failed once in `git_utils::tests::status_does_not_hand_the_caller_stdout_to_the_child` with `Os { code: 26, kind: ExecutableFileBusy, message: "Text file busy" }`. Main passed the same test, and a rerun of only that job passed.

The race: a test writes a script, so this process holds a write descriptor to it for a moment. If another test thread forks in that moment, the forked child keeps a copy of the descriptor until it execs. Exec'ing the script while any process holds it open for writing fails on Linux with ETXTBSY.

On main 884a33d3, a stress run in a `rust:1.89` Linux container (`git_utils::tests --test-threads=16`, 300 rounds) failed 9 rounds: 7 in the status test and 2 in the descendant-pipe test, all ETXTBSY.

Leif asked to fix every test with the write-then-exec pattern (2026-11-05 18:29). Six do it:

| Test | How the script runs | Fix |
|---|---|---|
| `git_utils::tests::status_does_not_hand_the_caller_stdout_to_the_child` | test execs it | run as `sh <path>` |
| `git_utils::tests::output_returns_while_a_descendant_holds_the_pipe` | test execs it | run as `sh <path>` |
| `change_tests::substantive_plan::bounded_git_read_returns_while_a_descendant_holds_the_output_pipe` | test execs it | run as `sh <path>` |
| `github::tests::drift_issue_capture_sanitizes_untrusted_title_and_markdown_arguments` (fake `gh`) | product code execs `gh` from `PATH` | write it from a child `sh` |
| `github::tests::token_present_read_list_verify_and_import_paths_never_spawn_gh` (fake `gh`) | product code execs `gh` from `PATH` | write it from a child `sh` |
| `tests/integration/commands/issues.rs` `issues_create_runs_normal_drift_creation_for_stable_snapshots` (fake `gh`) | the `specsync` binary execs `gh` from `PATH` | write it from a child `sh` |

For a fake `gh` the code under test execs it by name, so `sh` cannot be put in front of it. Writing the file from a child `sh` process means this test process never holds a write descriptor to it, so no fork can inherit one.

Two other places that set mode `0o755` are not write-then-exec and are left alone: `semantic_application_resolves_registry_backed_canonical_paths.rs:3093` (`tool.sh` is only hashed) and `substantive_plan.rs:3405` (`context.md` is only hashed).
