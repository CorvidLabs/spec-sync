---
change: cut-git-process-spawns-per-specsync-command-and-run-the-release-candidate-suite-with-cargo-nextest-so-macos
artifact: context
---

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
