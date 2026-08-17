---
change: CHG-0135-register-20-integration-tests-that-never-compiled-and-guard-against-orphaned-tes
artifact: tasks
---

# Tasks

- [x] Establish the file truly never ran, by evidence rather than assertion: count `.rs`
      files in `tests/integration/` against `#[path]` declarations in `tests/integration.rs`
      and show the set difference is exactly `regression_w1.rs`.
- [x] Confirm from git history that it was never registered in the commit that introduced it
      (`9a00223b`) or any commit since.
- [x] Register it with `#[path = "integration/regression_w1.rs"] mod regression_w1;`.
- [x] Remove the single unused import the compiler flags; change nothing else to compile.
- [x] Run the 20 tests and classify every failure as (a) a product bug to file, (b) an
      assertion that was always wrong, or (c) behaviour that deliberately changed.
- [x] File the product defects separately: #605, #606, #607. Do not fix product code here.
- [x] Fix the `report` fixture (#607) so both `report` tests measure the flag again, and
      prove the fixture discriminates rather than merely goes green.
- [x] Rewrite the miscalibrated `deps` assertion to assert the real dedupe signal
      (`Edges: 1`), keeping the 1/2/3-repeat control that proved dedupe works.
- [x] Add a self-documenting pin for #606 that states its own inversion condition, and
      verify the pin actually fires by temporarily neutering the duplicate emission.
- [x] Add `every_integration_test_file_is_registered` asserting set equality both
      directions, placed inline in `tests/integration.rs` so it cannot itself be orphaned.
- [x] Verify the guard can fail: plant a stray `.rs`, confirm rc=101 and that the message
      names it, then remove the probe.
- [x] Count `#[test]` markers before and after to prove no test was added, removed, renamed
      or `#[ignore]`d.
- [x] `cargo test` and `cargo clippy -- -D warnings` green.
- [x] CHANGELOG entry under `## [Unreleased]`.
