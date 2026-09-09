---
change: use-current-workflow-fixtures-in-verification-persistence-tests
artifact: context
---

# Context

CI on #774 failed `exact_and_multiple_verification_persistence_commits_remain_current` and `scoped_review_persistence_commit_keeps_verification_current`.

Both tests build records through `verification_history_fixture` → `completed_no_spec_record`, which persists `workflow_version = 1`. The previous package correctly taught v1 `Verifying` to name `verify` then `accept` then `archive`. These tests still asserted the v2 `review` / `finalize` strings, so they failed.

The tests are about verification persistence and scoped review — v2 behavior. The v1 close-out is covered by `workflow_v1_verifying_next_action_names_verify_accept_archive`. Switch the shared fixture to `completed_no_spec_current_record`. Do not change product `next_action`.
