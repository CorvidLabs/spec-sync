---
id: run-the-git-utils-test-scripts-through-sh-so-a-concurrent-fork-cannot-fail-their-exec-with-etxtbsy
state: draft
type: bug_fix
base_commit: 884a33d3c0cb95680dae786ff25688569ee16c96
---

# Run the git_utils test scripts through sh so a concurrent fork cannot fail their exec with ETXTBSY

## Intent

Run the git_utils test scripts through sh so a concurrent fork cannot fail their exec with ETXTBSY

## Affected Canonical Specs

- `git_utils`
- `change`
- `github`

## Acceptance Criteria

- Every test that runs a script it just wrote stops holding a write descriptor to that file at exec time: the three that exec their own script run it as `sh <path>`, and the three that put a fake `gh` on `PATH` write it from a child `sh` process. Assertions are unchanged.
- In a `rust:1.89` Linux container, 300 rounds of each affected test group (`git_utils::tests`, the bounded-runner tests in `change_tests`, `github::tests`, and the `issues` integration tests) fail 0 rounds with the change. `git_utils::tests` failed 9 of 300 on main 884a33d3.
- No gate, timeout or assertion is loosened.

## No-spec Rationale

Test harness only: two git_utils unit tests exec a script they just wrote, which Linux can refuse with ETXTBSY while another test thread forks. Running the script through sh changes no public behavior, requirement or spec text.
