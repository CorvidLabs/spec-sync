---
change: CHG-0030-satisfy-rust-1-95-clippy-for-the-cargo-lifecycle-verification-guard
artifact: plan
---

# Plan

1. Replace the negated `is_some_and` expression with the Clippy-preferred `is_none_or` equivalent.
2. Run formatting, focused behavior tests, full lifecycle verification, and the Fledge Trust lane.
