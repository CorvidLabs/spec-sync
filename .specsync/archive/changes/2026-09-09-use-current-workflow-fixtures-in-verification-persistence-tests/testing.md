---
change: use-current-workflow-fixtures-in-verification-persistence-tests
artifact: testing
---

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
