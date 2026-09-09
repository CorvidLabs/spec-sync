---
change: v1-verification-freshness-test-expects-verify-accept-archive-next-action
artifact: testing
---

# Testing

Fail: `cargo test --test integration -- verification_freshness_status_and_check_are_environment_independent` left = v1 verify/accept/archive, right = v2 review.

Pass: that test, plus `cargo test --test integration -- change::` if time allows. Unit `--bin specsync` already 2500 passed.
