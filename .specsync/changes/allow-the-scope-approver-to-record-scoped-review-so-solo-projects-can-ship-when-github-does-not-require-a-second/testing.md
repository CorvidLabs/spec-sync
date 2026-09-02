---
change: allow-the-scope-approver-to-record-scoped-review-so-solo-projects-can-ship-when-github-does-not-require-a-second
artifact: testing
---

# Testing

- `scoped_review_requires_an_independent_passing_verdict` must succeed when the reviewer is the definition approver (case-insensitive).
- Invalid reviewer claims still fail (non-ASCII identity).
- Block then pass still appends attempt history.
- `cargo test change::`
- `cargo test`

## Requirement evidence

| ID | Evidence |
|----|----------|
| REQ-change-046 | `scoped_review_allows_the_scope_approver_and_records_pass_and_block_verdicts`, `persisted_scoped_review_allows_scope_approver_as_reviewer` |
| REQ-agents-006 | `src/agents.rs` generated skill text; agents install templates |
| REQ-cmd-change-015 | `src/commands/change.rs` next-action copy; `tests/integration/change.rs` |
