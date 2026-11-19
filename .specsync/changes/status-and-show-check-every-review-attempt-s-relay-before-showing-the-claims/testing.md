---
change: status-and-show-check-every-review-attempt-s-relay-before-showing-the-claims
artifact: testing
---

# Testing

- Unit `recorded_claims_refuse_a_damaged_earlier_review_attempt_relay` in
  `src/change_tests/relayed_decision.rs`: with a valid latest review, an earlier attempt naming
  `Mallory` and then one with a `null` relay are each refused, naming `repair-relays` (fails on
  main: the claims are returned); a review with no attempt history still yields its claim.
- The existing relay, repair-relays and status tests still pass.
