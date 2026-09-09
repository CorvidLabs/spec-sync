---
change: v1-verification-freshness-test-expects-verify-accept-archive-next-action
artifact: context
---

# Context

CI `cargo test --verbose` now finishes the unit harness (2500 passed) and fails one integration test: `verification_freshness_status_and_check_are_environment_independent`.

The fixture writes `sdd.json` `"version": 1` and runs `change verify`. After the v1 Verifying next_action change it correctly reports verify → accept → archive. The test still expected v2 `review` / `check` strings.

Update the three expected next_action strings. Do not change product code.
