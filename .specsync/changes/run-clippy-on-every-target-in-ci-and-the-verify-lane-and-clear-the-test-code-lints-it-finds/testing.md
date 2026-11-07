---
change: run-clippy-on-every-target-in-ci-and-the-verify-lane-and-clear-the-test-code-lints-it-finds
artifact: testing
---

# Testing

| Case | Expected | How |
|---|---|---|
| A clippy step or task without `--all-targets` | The test fails and names the recovery | `test_every_ci_clippy_step_lints_every_target_and_denies_warnings` and `test_every_fledge_clippy_task_lints_every_target_and_denies_warnings` fail on the unmodified `ci.yml` and `fledge.toml` |
| A clippy command that lets lints through (`-A`, joined `-A…`, `--allow`, `--cap-lints`, no `-D warnings`) | Caught | `test_the_problem_check_catches_each_weakening` feeds each form to the rule; the joined form failed before its Codex round-1 fix |
| The lint gate is removed | The test fails | CI's `test` job must run clippy; `lint` must exist and run in `verify` and `ci` (`test_the_verify_and_ci_lanes_run_the_lint_task`) |
| The test stops running | The test fails | `test_this_test_runs_through_fledge_and_ci` |
| The test code lints clean | `cargo clippy --all-targets -- -D warnings` exits 0 | Fails with 36 errors on ab688246 and passes at the tip; `cargo test` and `cargo clippy -- -D warnings` still pass |
| No lint is weakened | No new `allow` or `expect` | `git diff origin/main` adds no `allow(`/`expect(` attribute; the `allow(clippy::` count stays at 28 |

Failing first: on the unmodified tree the lockstep test fails 3 tests (CI step, fledge task, not wired in), and `cargo clippy --all-targets -- -D warnings` exits 101 with 36 errors.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-github-028 | `.github/scripts/test-clippy-all-targets.py`: `test_every_ci_clippy_step_lints_every_target_and_denies_warnings`, `test_every_fledge_clippy_task_lints_every_target_and_denies_warnings`, `test_the_verify_and_ci_lanes_run_the_lint_task`, `test_the_problem_check_catches_each_weakening`, `test_this_test_runs_through_fledge_and_ci`; `cargo clippy --all-targets -- -D warnings` exit 0 at the tip |
