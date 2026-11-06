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
