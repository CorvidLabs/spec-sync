# Lesson bundle — use-current-workflow-fixtures-in-verification-persistence-tests

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Use current-workflow fixtures in verification persistence tests
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change_tests.rs
- **Acceptance**: verification persistence tests that assert v2 review or finalize next_action build current-workflow records; workflow_v1_verifying_next_action_names_verify_accept_archive still passes; cargo test exact_and_multiple_verification_persistence_commits_remain_current and scoped_review_persistence_commit_keeps_verification_current pass

## Evidence

- Verification commit: `05105e8193a701f8b7bc6af57c10d7059c9cc261`
- Base commit: `e9c1cf2d41289d84db4214fe26b42cf2f3ea59ff`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

CI on #774 failed `exact_and_multiple_verification_persistence_commits_remain_current` and `scoped_review_persistence_commit_keeps_verification_current`.

Both tests build records through `verification_history_fixture` → `completed_no_spec_record`, which persists `workflow_version = 1`. The previous package correctly taught v1 `Verifying` to name `verify` then `accept` then `archive`. These tests still asserted the v2 `review` / `finalize` strings, so they failed.

The tests are about verification persistence and scoped review — v2 behavior. The v1 close-out is covered by `workflow_v1_verifying_next_action_names_verify_accept_archive`. Switch the shared fixture to `completed_no_spec_current_record`. Do not change product `next_action`.

## From the change's testing.md

# Testing

Fail then pass:

```
cargo test --bin specsync -- exact_and_multiple_verification_persistence_commits_remain_current scoped_review_persistence_commit_keeps_verification_current
```

failed with left = v1 `verify`/`accept`/`archive`, right = v2 `review` / `finalize`.

After the fixture switch those two pass. Also:

```
cargo test --bin specsync -- workflow_v1_verifying_next_action_names_verify_accept_archive
cargo test --bin specsync -- verification_history_fixture read_scope_memoizes_repeated_summary_git_lookups scoped_review_persistence persisted_scoped_review mixed_persistence malicious_state_contract evidence_only_merge
```

v1 next_action must still name verify then accept then archive and must not name `change check` or `change finalize`.

## Where these lessons go

- `specs/change/context.md`
