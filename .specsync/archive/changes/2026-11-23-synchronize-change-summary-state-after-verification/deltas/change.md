---
change: synchronize-change-summary-state-after-verification
artifact: delta
module: change
---

## ADDED

### REQUIREMENT REQ-change-check-summary-state-001

When a scoped check records a completed verification, SpecSync SHALL publish its latest evidence, attempt ledger, and Verifying state in both state.json and change.md in one recoverable transaction.

Acceptance Criteria

- A passing verification and a completed spec-to-code failure both leave state.json, the change.md frontmatter, and change status at verifying.
- Rechecking an active change repairs an older implementing summary and preserves surrounding Markdown notes.
- A pre-verification refusal keeps the current lifecycle state and refusal-only evidence behavior. A publication error restores the prior state and evidence files.
- check --commit commits the synchronized summary and passing evidence that still allows scoped review.
