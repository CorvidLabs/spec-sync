# Lesson bundle: cover-the-send-back-wiring-in-materialization-review-binding-and-the-project-check

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Cover the send-back wiring in materialization, review binding and the project check
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/collect_requirement_ids.rs, src/change/historical_definition_digest_matches.rs
- **Acceptance**: A sent-back change converges on its own earlier canonical text, a send-back-cancelled approval binds no review, accept refuses orphaned applied text, and the project check reports a deleted send-back, as REQ-change-123 and send_back_for_rescope.rs already prove.

## Evidence

- Verification commit: `f85feada51dafd727f0b36cea448722329de0963`
- Base commit: `b7a80fc2543308003303fcf179c9a5a700eb47a7`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

PR #846 CI's Lifecycle gate refused `src/change/collect_requirement_ids.rs` and `src/change/historical_definition_digest_matches.rs` as changed paths covered by no change. Both carry review-fix wiring for `change send-back` (REQ-change-123): materialization converges after a send-back, a cancelled approval binds no review, accept refuses orphaned applied text, and the project check reports a deleted send-back. The archived send-back change's `affected_paths` omitted them; this change covers them without altering any contract text.

## From the change's testing.md

# Testing

Covered by `src/change_tests/send_back_for_rescope.rs` (REQ-change-123 evidence in specs/change/testing.md); `cargo test --bin specsync -- send_back sent_back` passes on this tip.

## Where these lessons go

- `specs/change/change.3md`
