---
change: allow-the-scope-approver-to-record-scoped-review-so-solo-projects-can-ship-when-github-does-not-require-a-second
artifact: tasks
---

# Tasks

- [x] Stop refusing same-actor scoped review in `record_scoped_review_with_verdict` and attempt-history validation.
- [x] Update scoped-review tests so the approver can record a passing review.
- [x] Amend REQ-change-046, change.spec.md contract item 6, ADOPTING.md, AGENTS.md, and generated agent skill text.
- [x] `cargo test change::` green. Full `cargo test` runs in `specsync change check`.
