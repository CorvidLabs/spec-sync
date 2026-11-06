---
change: cover-the-one-test-fixture-line-that-merging-969-into-900-needs
artifact: testing
---

# Testing

The line changes no behavior. It gives #969's review fixture the field every `ScopedReviewRecord` now has, set to "no relay". The tests that already cover the record and its status:

| Criterion | Test |
| --- | --- |
| SHIP-2.e | `a_relayed_review_finalizes_and_its_review_digest_leaves_the_relay_out` and `an_approval_and_a_review_without_a_relay_omit_the_record` in `tests/integration/relayed_decision.rs`: a review carries the relay record and finalizes, and a review without one omits the key. `the_review_digest_leaves_the_relay_out_and_keeps_existing_digests` in `src/change_tests/relayed_decision.rs` shows a record without a relay keeps its digest. |
| CHANGE-13.b | `status_shows_the_approval_and_review_claims_with_the_relay_and_its_boundary` in `tests/integration/relayed_decision.rs`: status shows each relay beside the approval or review it belongs to. |

`a_send_back_answers_review_order_by_attempt_count_and_falls_back_to_time` passes with the line, as do the other send-back tests (9 passed), and the test binary does not compile without it. The verify lane passed on a20353b9 (2870 unit, 2 ignored; 514 integration). After the check commit it runs again with `change audit --strict` and pre-push, and the audit must pass on the archive tip before push.
