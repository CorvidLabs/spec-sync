---
change: run-the-git-utils-test-scripts-through-sh-so-a-concurrent-fork-cannot-fail-their-exec-with-etxtbsy
artifact: testing
---

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
