---
change: make-a-same-second-review-test-deterministic-and-assert-on-the-review-ledger-instead-of-record-inequality
artifact: testing
---

# Testing

1. Red first: `PinnedTestClock::at(..)` in the #969 unit test pins every record to one second. It fails on every run at `assert_ne!(fresh, earlier)`: 3 of 3 runs on 5181fb2f plus this change's hook.
2. Fix: replace the inequality with "the attempt ledger grew by exactly one, its last entry is `fresh`, and `fresh` is current". Then the test passes pinned, on repeated runs.
3. The other assertions in the test are unchanged and pass in the same second. The send-back, scoped-review and #914/#960 tests stay green.
4. `specsync change check <id> --commit`, `specsync change audit --strict`, `fledge lanes run verify` on pinned 1.89, pre-push.

## Results

- Red: pinned, the test failed every run at `assert_ne!(fresh, earlier)`: 3 of 3 on 5181fb2f, and again after merging main through #969.
- Fixed: it asserts that the ledger grew by one and its last entry is the fresh, current review. It passes 3 of 3 pinned and once unpinned.
- The 17 send-back and scoped-review unit tests are green.
