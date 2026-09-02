---
change: allow-the-scope-approver-to-record-scoped-review-so-solo-projects-can-ship-when-github-does-not-require-a-second
artifact: design
---

# Design

Remove the case-insensitive `reviewer == scope_approver` refusal in `record_scoped_review_with_verdict` and the matching attempt-history validator. Keep reviewer-claim validation, append-only attempt history, pass/block verdicts, and GitHub as merge authority. Distinct reviewers remain allowed. Same-actor review is allowed. Generated agent skill text, AGENTS.md, and ADOPTING.md drop the "pick a second identity" instruction.
