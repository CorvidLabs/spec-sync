# Lesson bundle: cut-git-process-spawns-per-specsync-command-and-run-the-release-candidate-suite-with-cargo-nextest-so-macos

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Cut git process spawns per SpecSync command and run the release-candidate suite with cargo nextest so macOS qualification fits 45 minutes
- **Kind**: Feature
- **Specs**: git_utils, change, cmd_change
- **Paths**: src/git_utils.rs, src/change/validate_loaded_change.rs, src/change/consider_accepted_evidence_anchor.rs, src/change/historical_definition_digest_matches.rs, src/change/read_cache.rs, src/change/sdd_version.rs, src/commands/change.rs, src/change_tests/, tests/integration/change.rs, fledge.toml, .github/workflows/release.yml, docs/RELEASING.md, CHANGELOG.md, specs/git_utils/, specs/change/, specs/cmd_change/, hi/speed.md, .chough/, .agents/skills/chough-workflow/, INTENT.md
- **Acceptance**: Within one SpecSync command, git is asked each repository fact the command does not itself change (HEAD commit, show-prefix, work-tree and shallow state, origin default branch, index location, fsmonitor and core settings) at most once, and command output is byte-identical to before.
- **Acceptance**: A memoized fact is dropped after the command itself commits, writes the index or moves a ref, so the next read asks git again.
- **Acceptance**: Committed file contents and tree listings that one command reads at fixed commits come from one `git cat-file --batch` process per repository instead of one `git show` or `git ls-tree` per file.
- **Acceptance**: A test bounds git process spawns per lifecycle command, and the slowest lifecycle integration test (`eight_workflow_v2_changes_finalize_in_their_originating_prs_without_duplicate_product_ci`) spawns at most 40% of its 15,563-process baseline.
- **Acceptance**: The release-candidate lane runs the same test set with a checksum-pinned `cargo-nextest`; no test is removed, ignored, filtered or loosened, and the two tests that are `#[ignore]` today stay the only ignored ones.
- **Acceptance**: Before and after timings and spawn counts are recorded in `testing.md`.

## Evidence

- Verification commit: `113893d85fef9faeabdc91f3ae87c0ca8e374d1a`
- Base commit: `67aa7dc25ab8f1ddb732b906b933bb778e2c86aa`
- Verified by: `specsync check --spec change --spec cmd_change --spec git_utils`

## From the change's context.md

# Context

v6.2.0-rc.1's macOS qualification was cancelled at the 45-minute job limit twice (run 41971049002), so #860 raised the limit to 75 minutes as a stopgap and issue #861 tracks the cause. v6.2.0-rc.2 then qualified on macos-15 in 49 minutes (run 42035620884): test build 4m06s, unit tests 2025s, integration tests 338s, release build 5m01s. Ubuntu ran the same unit tests in 214s and the integration tests in 58s.

Profiling on a local 20-core Mac at main 67aa7dc2 reproduced it. The cost is git process spawning, not computation:

- Under cargo nextest all 3242 tests pass in 811s wall (cargo test: about 2000s), with sys 4364s greater than user 2398s.
- 152 of the 3242 tests carry 92% of test time.
- The slowest test, `eight_workflow_v2_changes_finalize_in_their_originating_prs_without_duplicate_product_ci`, takes 343s alone and spawns 15,563 git processes over about 150 SpecSync invocations.
- About half of those spawns re-ask facts that do not change within a command: `rev-parse --verify HEAD` 1258, `rev-parse --show-prefix` 1066, `symbolic-ref --short refs/remotes/origin/HEAD` 756, `rev-parse --verify origin/main` 752 and `origin/main^{commit}` 712, `--is-inside-work-tree` 718, `--is-shallow-repository` 608, and about 448 each of `--shared-index-path`, `--git-path index`, `config --get core.fsmonitor` and the core-settings regexp.
- About 2,100 more read one blob or tree at a time: `show <sha>:<path>` 1253, `ls-tree` 512, `show HEAD:<path>` 328.

Ruled out: an optimized test build (opt-level 2 ran the slowest test in 386s against 343s), and a larger runner (it would hide the cost from users, who pay the same spawns in real lifecycle commands).

Leif approved the intent (SPEED-2.b.1, SPEED-2.b.2) and this approach on 2026-11-05 at 09:21 MST in the Herdr orc tab.

## From the change's design.md

# Design

1. Repository-fact memo in `git_utils`. A per-process memo keyed by the canonical repository root caches the answers to: HEAD commit (`rev-parse --verify HEAD`), `--show-prefix`, `--is-inside-work-tree`, `--is-shallow-repository`, the origin default branch and its commit, `--git-path index`, `--shared-index-path`, `core.fsmonitor` and the core settings regexp. Callers in `change/` and `commands/change.rs` ask the memo instead of spawning. Every SpecSync code path that commits, writes the index, runs `update-ref` or switches branches calls one invalidation function for that root, so the next read asks git again. Failures are not memoized (git_utils keeps its safe-default contract).
2. Batched object reads. One lazily started `git cat-file --batch` child per repository root serves `<commit>:<path>` blob reads and tree listings at fixed commits. Reads at `HEAD:` resolve HEAD through the memo first, so a commit inside the command invalidates correctly. The child is closed when the command ends. Missing objects return the same "absent" answer the per-file `git show` path gives today.
3. Spawn bound. A test-only spawn counter around `git_cmd` asserts a ceiling per lifecycle command, extending the existing `*_git_processes_do_not_grow_*` tests, so the saving cannot silently regress.
4. Release-candidate lane. `fledge.toml` gains a `test-nextest` task (`cargo nextest run --no-fail-fast` with the default profile, no filters) and `lanes.release-candidate` uses it in place of `test`. `release.yml` installs a pinned cargo-nextest release asset verified by sha256, the same way it installs Fledge. `lanes.verify`, `lanes.ci` and Ubuntu CI keep `cargo test`, so both runners stay exercised.

No public CLI, output or file format changes. The memo and reader live only for one process.

## From the change's testing.md

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

## Where these lessons go

- `specs/git_utils/git_utils.3md`
- `specs/change/change.3md`
- `specs/cmd_change/cmd_change.3md`
