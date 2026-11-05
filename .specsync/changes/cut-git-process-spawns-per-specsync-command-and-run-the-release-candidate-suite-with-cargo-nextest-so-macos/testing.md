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

## After (cbb4beb9..HEAD, local 20-core Mac)

| Measure | Before (67aa7dc2) | After |
|---|---|---|
| Git spawns, one-change fixture: new / answer / approve | 63 / 71 / 89 | 29 / 15 / 32 |
| check / status / review / finalize | 388 / 66 / 323 / 739 | 110 / 53 / 176 / 287 |
| Slowest lifecycle test, git spawns | 15,563 | 5,996 (38.5%, target at most 6,225) |
| Slowest lifecycle test, alone | 343s | 95s |
| Full suite under cargo nextest | 811s wall, 3242 passed | 426s wall, 3251 passed, 2 skipped |

- The 2 skipped tests are the same two `#[ignore]` tests as before; nine tests were added (7 git_utils unit tests, the live-read test, the spawn-ceiling integration test).
- Race checks stay live: evidence inspection, repository identity and the adoption snapshot ask git every time (50 race, rollback and fail-closed tests plus the fsmonitor and ident tests pass: 107 in one targeted run).
- `cargo clippy -- -D warnings` passes. `--all-targets` reports lints only on pre-existing lines outside this change.
- The macOS CI qualify time is measured on the next release candidate; the timeout stays 75 until then (Leif, 2026-11-05).
- Observed during testing, not caused by this change: `fsmonitor_valid_and_ident_conversion_fail_closed` hung once in a full run when other agents ran the same test concurrently; its own `git update-index --fsmonitor-valid` setup blocked on shared fsmonitor daemons. Alone it passes in 5s, and it passed in the final full run.
