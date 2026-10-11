# Lesson bundle — the-handoff-must-not-claim-a-commit-it-did-not-check

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: The handoff must not claim a commit it did not check
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/historical_definition_digest_matches.rs, src/change_tests/canonical_resolution_tolerates_inert_legacy_registry_stub.rs
- **Acceptance**: The verifying handoff says verification is current for this working tree and a fresh session here resumes at the independent review. It does not say the implementation is committed. The approved and implementing handoff does not say the tree is committed.

## Evidence

- Verification commit: `f8d6094ea29f8e8149b90b79183c84a484d53e6d`
- Base commit: `68fab607785719474db1c3573b6d25cce9309e95`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

A dogfood of the split on main approved a no-spec change, ran `change check` without `--commit`, and left the change directory untracked. The handoff still said the implementation was committed and a fresh session could resume at review. A new clone of HEAD would not contain that change.

`HandoffSignals` records whether affected paths are dirty and whether verification matches this working tree. It does not record that a commit happened. Evidence under `.specsync/` stays uncommitted until finalize on purpose. The reason may claim only those signals.

## From the change's testing.md

# Testing

`handoff_verifying_follows_evidence_currency` already requires "verification is current" and "independent review". It will also reject any reason that contains "committed".

`handoff_approved_and_implementing_with_a_clean_tree_are_safe` will reject a reason that contains "committed" and still require `` `change check` ``.

Run `cargo test --bin specsync -- handoff_verifying_follows_evidence_currency -- --exact` and `cargo test --bin specsync -- handoff_approved_and_implementing_with_a_clean_tree_are_safe -- --exact`. One filter per invocation.

## Where these lessons go

- `specs/change/change.3md`
