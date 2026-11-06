---
change: let-a-change-reviewed-before-its-send-back-be-reviewed-again-after-re-approval
artifact: testing
---

# Testing

Failing test first, through the built binary, under workflow v2.

1. Integration (`tests/integration/change.rs`), the #914 repro: approve, `check --commit`, review, send-back, re-answer the criteria, approve, `check --commit`.
   - Before the second review: `change status --json` reports the review stale with a next action naming `change review` (not unreadable), and `change ship` refuses.
   - The second `change review` records a pass, and `change ship`/`finalize` archive the change; `review-attempts.json` in the archive holds both attempts, the first unchanged.
   Must fail on main cd8cab5e at the second review.
2. Unit (`send_back_for_rescope.rs`): review, send-back, re-answer, re-approve, check-and-commit; on the real ledger the earlier review binds historically and not live; it loads unchanged, is `Stale`, and the fresh review is current and binds the post-send-back approval.
3. Existing send-back, scoped-review and committed-history tests unchanged and green.
4. `specsync change check <id> --commit`, `fledge lanes run verify` (pinned toolchain), pre-push.

## Results

Pending.
