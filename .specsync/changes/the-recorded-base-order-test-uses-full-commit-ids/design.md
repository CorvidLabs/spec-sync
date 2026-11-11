---
change: the-recorded-base-order-test-uses-full-commit-ids
artifact: design
---

# Design

The test's two records now carry full 40-hex commit ids, so `recorded_diff_base` still has to pick the
oldest change's base, as the test intends; the short ids it used before are no longer accepted.
