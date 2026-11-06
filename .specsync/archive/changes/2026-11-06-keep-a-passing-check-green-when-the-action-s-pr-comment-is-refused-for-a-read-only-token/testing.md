---
change: keep-a-passing-check-green-when-the-action-s-pr-comment-is-refused-for-a-read-only-token
artifact: testing
---

# Testing

- `config::a_comment_the_token_may_not_post_leaves_the_check_result_unchanged` (new) extracts the Post PR Comment `run:` script from `action.yml` and runs it in bash against a fake `gh`:
  - a 403 on the lookup, create or update must exit 0, print a `::warning` naming `pull-requests: write`, and write the markdown to `GITHUB_STEP_SUMMARY`;
  - a 500 must still fail the step;
  - success must still print `Posted SpecSync PR comment`.
  It fails before the fix: a 403 on the lookup exits nonzero.
- The existing `action_*` hardening tests in `tests/integration/config.rs` still pass.
- `fledge lanes run verify` before review.

## Results

- `config::a_comment_the_token_may_not_post_leaves_the_check_result_unchanged` failed first, committed as 680d9fe1: a 403 on the lookup exited nonzero. It now passes for a 403 on the lookup, create and update, for a 500 (still fails the step), and for posting and updating with a writable token.
- The first fix still failed the test. The lookup ran inside `$(...)`, so the refusal's `exit 0` ended only the subshell and the warning was captured as output. The lookup now writes to a file, and the warning goes to the step's own stdout on fd 3.
- The existing `config::action_*` hardening tests pass.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-github-024 | `config::a_comment_the_token_may_not_post_leaves_the_check_result_unchanged` (403 on lookup, create and update; 500; post; update) |
