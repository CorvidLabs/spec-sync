# Lesson bundle: the-recorded-base-order-test-uses-full-commit-ids

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: The recorded base order test uses full commit ids
- **Kind**: BugFix
- **Specs**: change, cmd_change
- **Paths**: src/change_tests/semantic_application_resolves_registry_backed_canonical_paths.rs, src/change/validate_loaded_change.rs, src/change/collect_requirement_ids.rs, src/commands/change.rs, src/change_tests/commit_identity_guards.rs
- **Acceptance**: Only the oldest active change's recorded base is considered as the coverage diff base, and one that is not a full commit id falls back to the no-record base (two-record test); the order test uses full commit ids; status evidence shares is_canonical_commit_id

## Evidence

- Verification commit: `d9524a6f96f84a1afb68ac33496203174ec8094b`
- Base commit: `f3605f2c9a97af46d8f0cdf0aa46825b70acf1c2`
- Verified by: `specsync check --spec change --spec cmd_change`

## From the change's context.md

# Context

CI on #1052's archive tip f3605f2c failed one unit test whose fixture used short fake base ids, which the
full-commit-id rule ignores. corvid-agent's review of #1052 also asked that only the oldest change's base
count, as the acceptance criterion says, and that the status evidence share the commit-id helper.

## From the change's design.md

# Design

- `recorded_diff_base` considers only the oldest active change's `base_commit`, and uses it only when it
  is a full commit id. Any other value falls back to the no-record base (`HEAD~1...HEAD`, or `HEAD`), never
  to a later change's base, as the acceptance criterion of the full-commit-id change says.
- `is_canonical_commit_id` is `pub(crate)` and `change status` evidence uses it instead of a copy.
- It accepts SHA-1 ids only. A SHA-256 repository would see every recorded commit treated as absent; that is
  the existing recording format, and noted here for when SpecSync records SHA-256 ids.
- The order test's records carry full commit ids.

## From the change's testing.md

# Testing

`recorded_base_uses_oldest_change_order_not_hash_order` (CI's `test` job failed on it at f3605f2c) and the
two-record case in `a_recorded_commit_that_is_not_a_full_commit_id_is_never_handed_to_git` pass.

## Where these lessons go

- `specs/change/change.3md`
- `specs/cmd_change/cmd_change.3md`
