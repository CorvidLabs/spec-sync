---
change: cut-git-process-spawns-per-specsync-command-and-run-the-release-candidate-suite-with-cargo-nextest-so-macos
artifact: testing
---

# Testing

## Baseline (before, main 67aa7dc2)

- CI macos-15 qualify, v6.2.0-rc.2 (run 42035620884): 49m04s total; unit tests 2025s, integration 338s.
- Local 20-core Mac, cargo nextest, full suite: 811s wall, 3242 passed, 2 skipped.
- Slowest test alone: 343s, 15,563 git spawns.

## Plan

- `cargo test` and `cargo nextest run` both pass with the same test count and the same two ignored tests.
- Spawn count for the slowest test, measured with the same PATH shim, is at most 6,225 (40% of 15,563).
- Memo invalidation tests: a command that commits, writes the index or moves a ref reads fresh facts afterwards.
- Batched reads match per-file reads, including missing paths and missing objects.
- `fledge lanes run verify` passes; `fledge lanes run release-candidate` passes locally under nextest.

## After

(Filled in during check.)
