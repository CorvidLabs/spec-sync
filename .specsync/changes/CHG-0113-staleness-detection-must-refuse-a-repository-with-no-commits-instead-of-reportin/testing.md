---
change: CHG-0113-staleness-detection-must-refuse-a-repository-with-no-commits-instead-of-reportin
artifact: testing
---

# Testing

## Strategy

A guard that refuses is easy to make too broad — a version that refused every repository
would satisfy the filed defect exactly. So the assertions bracket it: the degenerate state
must be refused, **and** the working state must be untouched, **and** the two refusal
reasons must stay apart.

## Verified by hand — all three states

| state | result |
|---|---|
| git repo, **zero commits** | `Error: Repository has no commits — staleness detection requires git history.`, exit 1; JSON `error: "repository has no commits"` |
| **the same repo** after one commit | `✓ All specs are up to date with their source files.`, exit 0 |
| not a git repository | `Error: Not a git repository — staleness detection requires git history.`, exit 1 |

The middle row is the load-bearing one: it is the *same fixture*, one commit later, so the
guard demonstrably keys on the absence of history rather than on anything else about the
project.

The third row matters because collapsing both causes into one message would have been less
code. A user who ran this inside a repository they had just created needs to know it is the
commits that are missing, not the repository.

## Regression surface

2210 unit and 331 integration tests pass unchanged. `has_commits` is additive, and the
staleness guard only widens a branch that already existed.

## Note on an earlier run of this suite

An earlier verification of this change reported green, but files were stashed out of the
working tree while it was running, so the result could not be attributed to the code under
test. It was discarded rather than reported, and the suite above is a clean re-run against
an untouched tree.

## Not covered

No unit test asserts the new wording. `cmd_stale` has no output-test harness in this
change's scope; behavioural pinning belongs in the sandbox alongside drill 040, which
already covers this class.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-git-utils-002 | `cargo test` (2210 + 331, 0 failures); `has_commits` is exercised by the three fixtures above, which cover a repository with history, one with an unborn HEAD, and a non-repository |
| REQ-cmd-stale-002 | The three hand-verified states: zero commits refused with its own reason and exit 1, non-repository refused with a different reason and exit 1, and the same repository one commit later reporting normally at exit 0 — which is what confines the guard to a genuinely absent history |
