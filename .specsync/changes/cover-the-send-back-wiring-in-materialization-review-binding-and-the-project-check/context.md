---
change: cover-the-send-back-wiring-in-materialization-review-binding-and-the-project-check
artifact: context
---

# Context

PR #846 CI's Lifecycle gate refused `src/change/collect_requirement_ids.rs` and `src/change/historical_definition_digest_matches.rs` as changed paths covered by no change. Both carry review-fix wiring for `change send-back` (REQ-change-123): materialization converges after a send-back, a cancelled approval binds no review, accept refuses orphaned applied text, and the project check reports a deleted send-back. The archived send-back change's `affected_paths` omitted them; this change covers them without altering any contract text.
