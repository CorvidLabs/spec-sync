---
change: require-a-fresh-review-after-a-send-back-even-when-the-scope-comes-back-unchanged
artifact: research
---

# Research

- `definition_approver_bound_from` matches `approval.digest == review.contract_digest` over the approvals after `send_back_cancelled_prefix`, so a byte-identical re-approval is found the same way the cancelled one was.
- `ScopedReviewRecord` has `timestamp` (seconds) and no reference to the approval it followed. `SendBackRecord` has `timestamp` and `cancelled_approvals`.
- Callers of `scoped_review_currency` / `scoped_review_is_current`: status and handoff summaries, `recorded_scoped_review_currency` (ship-status), and the finalize/ship gate. Archive integrity does not re-run currency.
- Archives on main with send-backs (4): in each, the recorded review is later than the last send-back, so the legacy timestamp fallback changes none of them.
