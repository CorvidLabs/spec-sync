---
change: the-release-tests-exercise-the-race-guards-in-the-shipped-profile
artifact: testing
---

# Testing

`cargo test --bin specsync release_race_guard` (debug) and `cargo test --release --locked
--bin specsync -- release_race_guard` both run the three tests.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| CI-11 | unit `release_race_guard_coverage_root_refuses_a_replaced_root`, `release_race_guard_generate_root_refuses_a_replaced_root`, `release_race_guard_mcp_read_root_refuses_a_replaced_directory` under `--release` |
