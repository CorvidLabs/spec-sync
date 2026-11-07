# Lesson bundle: make-a-same-second-review-test-deterministic-and-assert-on-the-review-ledger-instead-of-record-inequality

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Make a same-second review test deterministic and assert on the review ledger instead of record inequality
- **Kind**: BugFix
- **Specs**: change
- **Paths**: hi/ci.md, src/change/validate_loaded_change.rs, src/change_tests/send_back_for_rescope.rs
- **Acceptance**: With now() pinned to one second through a test-only clock (compiled only into test builds, so release builds behave exactly as before), a_review_recorded_before_a_send_back_never_counts_after_an_identical_reapproval fails on every run at assert_ne!(fresh, earlier) before the fix.
After the fix the test asserts that review-attempts.json grew by exactly one attempt and that its last entry is the fresh review and is current, and it passes on every run, pinned or not.
Every other assertion in the test is unchanged and still passes in the same second: the send-back records one cancelled review attempt, the earlier review is stale for the send-back reason, and the fresh review is current.
No product code path changes; no test is dropped or weakened (SPEED-2.b.2).

## Evidence

- Verification commit: `a7bc2cb68abafba7ce90bad517ad04cc1aa39e1a`
- Base commit: `5181fb2f6952ef086eed6cb0dff976b41123cd77`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Issue #977, from the Codex review of #969 (#960). `a_review_recorded_before_a_send_back_never_counts_after_an_identical_reapproval` ends with `assert_ne!(fresh, earlier)`. On the unchanged-scope path the second review has the same implementation commit, reviewer, verdict and contract, execution and workspace digests as the first, and `timestamp` has one-second resolution. So when both reviews land in the same second the records are equal and the test fails, though a second attempt was correctly appended. Review currency finds a review by its last position in `review-attempts.json`, so the product is right; only the assertion is.

Intent CI-7, as Leif chose it ("Broad"): "A failing test always means something is actually wrong, never that it ran at an unlucky moment."

`now()` reads the system clock directly, so the flake cannot be shown on demand today. A clock pin that exists only in test builds makes the same second a fixture instead of luck.

Out of scope: product code, other tests' timing (none found that compares two records that differ only by `timestamp`), and the review record schema.

## From the change's testing.md

# Testing

1. Red first: `PinnedTestClock::at(..)` in the #969 unit test pins every record to one second. It fails on every run at `assert_ne!(fresh, earlier)`: 3 of 3 runs on 5181fb2f plus this change's hook.
2. Fix: replace the inequality with "the attempt ledger grew by exactly one, its last entry is `fresh`, and `fresh` is current". Then the test passes pinned, on repeated runs.
3. The other assertions in the test are unchanged and pass in the same second. The send-back, scoped-review and #914/#960 tests stay green.
4. `specsync change check <id> --commit`, `specsync change audit --strict`, `fledge lanes run verify` on pinned 1.89, pre-push.

## Results

- Red: pinned, the test failed every run at `assert_ne!(fresh, earlier)`: 3 of 3 on 5181fb2f, and again after merging main through #969.
- Fixed: it asserts that the ledger grew by one and its last entry is the fresh, current review. It passes 3 of 3 pinned and once unpinned.
- The 17 send-back and scoped-review unit tests are green.

## Where these lessons go

- `specs/change/change.3md`
