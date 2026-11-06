# Lesson bundle: run-the-git-utils-test-scripts-through-sh-so-a-concurrent-fork-cannot-fail-their-exec-with-etxtbsy

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Run the git_utils test scripts through sh so a concurrent fork cannot fail their exec with ETXTBSY
- **Kind**: BugFix
- **Specs**: git_utils, change, github
- **Paths**: src/git_utils.rs, src/change_tests/substantive_plan.rs, src/github.rs, tests/integration/commands/issues.rs
- **Acceptance**: Every test that runs a script it just wrote stops holding a write descriptor to that file at exec time: the three that exec their own script run it as `sh <path>`, and the three that put a fake `gh` on `PATH` write it from a child `sh` process. Assertions are unchanged.
- **Acceptance**: In a `rust:1.89` Linux container, 300 rounds of each affected test group (`git_utils::tests`, the bounded-runner tests in `change_tests`, `github::tests`, and the `issues` integration tests) fail 0 rounds with the change. `git_utils::tests` failed 9 of 300 on main 884a33d3.
- **Acceptance**: No gate, timeout or assertion is loosened.

## Evidence

- Verification commit: `55267abe5b94bfa5397ab9d044069b758997716b`
- Base commit: `884a33d3c0cb95680dae786ff25688569ee16c96`
- Verified by: `specsync check --spec change --spec git_utils --spec github`

## From the change's context.md

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

## From the change's testing.md

# Testing

All runs use a `rust:1.89` Linux container, the test binary built from the tree under test, and `--test-threads=16`, 300 rounds each.

Baseline on main 884a33d3:

| Group | Failing rounds | Failures |
|---|---|---|
| `git_utils::tests` alone | 9 / 300 | status test 7, descendant-pipe test 2, all ETXTBSY |
| `git_utils::tests change::tests::bounded_git github::tests` together (58 tests) | 5 / 300 | status test 5, all ETXTBSY |
| integration `commands::issues` | 0 / 300 | none |

The bounded-runner test in `change_tests`, the two `github` tests and the `issues` integration test were not seen failing. They have the same write-then-exec shape and are fixed preventively, at Leif's request.

With the change (same container, binary rebuilt from this tree):

| Group | Failing rounds |
|---|---|
| `git_utils::tests` alone | 0 / 300 |
| `git_utils::tests change::tests::bounded_git github::tests` together (58 tests) | 0 / 300 |
| integration `commands::issues` | 0 / 300 |

No ETXTBSY occurred. The six tests keep their assertions; only how the script is written or started changed. Locally on macOS the six tests and `issues_create_runs_normal_drift_creation_for_stable_snapshots` pass.

## Where these lessons go

- `specs/git_utils/git_utils.3md`
- `specs/change/change.3md`
- `specs/github/github.3md`
