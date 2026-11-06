---
id: run-the-git-utils-test-scripts-through-sh-so-a-concurrent-fork-cannot-fail-their-exec-with-etxtbsy
state: implementing
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

- Both git_utils unit tests that run a script they just wrote run it through sh and keep their assertions unchanged, A Linux stress run of git_utils::tests that failed 9 of 300 rounds with ETXTBSY on main fails 0 of 300 rounds with the change, No gate or timeout or assertion is loosened

## No-spec Rationale

Test harness only: two git_utils unit tests exec a script they just wrote, which Linux can refuse with ETXTBSY while another test thread forks. Running the script through sh changes no public behavior, requirement or spec text.
