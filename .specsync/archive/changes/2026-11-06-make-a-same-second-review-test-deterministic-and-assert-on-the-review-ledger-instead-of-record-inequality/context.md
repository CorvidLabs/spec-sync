---
change: make-a-same-second-review-test-deterministic-and-assert-on-the-review-ledger-instead-of-record-inequality
artifact: context
---

# Context

Issue #977, from the Codex review of #969 (#960). `a_review_recorded_before_a_send_back_never_counts_after_an_identical_reapproval` ends with `assert_ne!(fresh, earlier)`. On the unchanged-scope path the second review has the same implementation commit, reviewer, verdict and contract, execution and workspace digests as the first, and `timestamp` has one-second resolution. So when both reviews land in the same second the records are equal and the test fails, though a second attempt was correctly appended. Review currency finds a review by its last position in `review-attempts.json`, so the product is right; only the assertion is.

Intent CI-7, as Leif chose it ("Broad"): "A failing test always means something is actually wrong, never that it ran at an unlucky moment."

`now()` reads the system clock directly, so the flake cannot be shown on demand today. A clock pin that exists only in test builds makes the same second a fixture instead of luck.

Out of scope: product code, other tests' timing (none found that compares two records that differ only by `timestamp`), and the review record schema.
