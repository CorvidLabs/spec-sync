---
change: require-a-fresh-review-after-a-send-back-even-when-the-scope-comes-back-unchanged
artifact: testing
---

# Testing

Failing test first, through the built binary, under workflow v2.

1. Integration (`tests/integration/change.rs`): `a_send_back_needs_a_fresh_review_even_when_the_scope_comes_back_unchanged`. Approve, `check --commit`, review, send-back, approve unchanged, `check` without `--commit`.
   - Status names `change review`.
   - ship-status reports the review stale.
   - `change ship` refuses and leaves the change active.
   - A fresh review then finalizes with both attempts, the first unchanged.
   - Fails on main 8a251d7a (status says `scoped_review_current: true` and `finalize`).
2. Unit (`send_back_for_rescope.rs`):
   - the send-back records the attempt count;
   - after an identical re-approval the earlier review is `Stale` for the send-back reason, and the fresh one is `Current`;
   - a legacy send-back without the count uses the strict timestamp rule.
3. The existing send-back, scoped-review, committed-history and #914 tests are unchanged and green.
4. `specsync change check <id> --commit`, `specsync change audit --strict`, `fledge lanes run verify` on pinned 1.89, pre-push.

## Results

Pending.
