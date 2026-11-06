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

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-git-utils-008 | git_utils tests a_fact_read_twice_in_one_scope_is_answered_once, a_write_inside_the_scope_forgets_every_fact, nothing_is_memoized_without_an_open_scope_or_after_it_closes, working_tree_reads_and_stdin_queries_are_never_facts, live_reads_ask_git_even_inside_a_scope |
| REQ-git-utils-009 | git_utils tests batched_reads_equal_per_file_reads_from_one_child, a_batched_head_read_after_a_commit_sees_the_new_commit, an_object_over_the_bound_falls_back_to_git |
| REQ-change-125 | integration test lifecycle_commands_stay_within_git_spawn_ceilings; slowest lifecycle test measured at 5,996 git spawns against the 6,225 bound (PATH shim count, recorded above) |
| REQ-cmd-change-028 | the full lifecycle CLI suite (tests/integration/change.rs, 479 tests) passes unchanged; ship_push_never_commits_an_unrelated_untracked_file and the check --commit tests exercise reads after the command's own commit; change_adopt_rolls_back_when_comparison_ref_moves_during_publication proves race checks still see a concurrent ref move |
