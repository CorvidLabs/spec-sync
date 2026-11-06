# Lesson bundle: archived-change-status-no-longer-walks-history-per-archive-or-per-workflow-v1-change-and-read-commands-take-a-timeout

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Archived change status no longer walks history per archive or per workflow-v1 change, and read commands take a timeout that names the phase and how to resume
- **Kind**: Feature
- **Specs**: change, cmd_change, cli_args, git_utils
- **Paths**: src/change/consider_accepted_evidence_anchor.rs, src/change/validate_loaded_change.rs, src/change/read_cache.rs, src/change/read_deadline.rs, src/change_tests/fast_archived_reads.rs, src/commands/change.rs, src/cli.rs, src/git_utils.rs, site/src/content/docs/cli.md, CHANGELOG.md
- **Acceptance**: A warm change status of an archived change spawns fewer than 3 more git processes when the project holds 6 more archived workflow-v1 changes (warm_archived_status_git_processes_do_not_grow_with_legacy_changes, failing first: 94 -> 106 on main)
- **Acceptance**: A cold archive-introduction index build spawns fewer than 3 more git processes after 4 more archive-introducing commits (one diff-tree --stdin), and an equivalence test pins the per-commit result
- **Acceptance**: Prefetch workers start from the command repository facts in a read-only seeded scope with no batch reader, so a cold archived status read adds at most 150 git processes for 4 more archives (was 250)
- **Acceptance**: Whether a workflow-v1 change existed at the trusted pre-v2 cutoff is decided from the cutoff commit tree and kept in the persistent read cache keyed by that commit; existing refusals are unchanged and an equivalence test pins the verdict
- **Acceptance**: change status, show, list and ship-status take --timeout <secs> (or SPECSYNC_TIMEOUT); on expiry they exit 124, write nothing, flush completed verdicts, and print the phase, progress and exact resume command, in text and JSON
- **Acceptance**: On demand (fledge run bench-archived-reads, with workflow-v1 changes in the fixture) the warm archived status is under 2 s on an unloaded Mac; cold and warm times on this repository are recorded before and after; the cold under-5 s budget is measured and reported, and any remaining gap goes to a named follow-up
- **Acceptance**: No gate, bound, timeout default or approval rule is weakened, and no existing assertion is loosened

## Evidence

- Verification commit: `47ac3bb0474d66326d6e4294fbdcc81c651b5cac`
- Base commit: `560af9f10e608ce4c506612b0d1d2ae13378807d`
- Verified by: `specsync check --spec change --spec cli_args --spec cmd_change --spec git_utils`

## From the change's context.md

# Context

S9 of `_audits/95-plan.md` (performance, with latency budgets and actionable timeout recovery). #854 and #855 were closed by #864; #831 asks for a cheap authenticated-archive read. Leif's intent SPEED-1.a is that an archived change's status returns in under 2 s on Raven, and SPEED-1.d asks for a Raven-sized benchmark.

Leif approved step 1 (measurement only) at 21:43 MST and this approach, the budgets and the timeout design at 22:35 MST on 2026-11-05, relayed by orc t18.

Leif widened the scope on 2026-11-05 at 23:24 MST, relayed by orc t18 ("Widen (C)"): prefetch workers open a same-root fact scope seeded with the parent's facts, are read-only, get no batch reader, and keep a per-worker object memo, with a cold guard. Specs git_utils and change. The send-back was recorded by the orc as 0xLeif.

## From the change's design.md

# Design

**A. One walk for the archive-introduction index.** Replace the per-commit `diff-tree` with one `git log --format=%x00%H --diff-filter=A --no-renames --name-only -z -m --root <refs> -- <archive root>`. Group the added paths by commit, and keep only `<archive root>/<package>/state.json`. `-m` lists a merge commit's additions against every parent, as `diff-tree -m` did. The same `--max-count` bound applies to commits. The `state.json` and `approvals.json` reads go through `run_git_bounded` with full object ids, which the batch reader from #861 already answers. The per-commit entries stay in the persistent cache, so a partly warm cache still helps.

**B. The cutoff tree decides the workflow-v1 check.** "The record existed at the cutoff" asks only about paths in the cutoff commit's tree, and the baseline history check has already proven that commit reachable. So one `ls-tree -r --name-only -z <cutoff> -- <changes root> <archive root>` per command, memoized, lists every candidate `state.json` at the cutoff. Each workflow-v1 change then checks its active path and any `<archive root>/*-<id>/state.json` in that list, reading the record at the cutoff as today. The yes/no verdict per `(cutoff, id)` goes into the persistent read cache. The cutoff is an immutable commit, so the entry never needs invalidating. A rewritten baseline names a different cutoff, which is a different key.

**Deadline.** A thread-local read deadline, set by the command layer from `--timeout` or `SPECSYNC_TIMEOUT`. The long loops (archive-introduction index, per-change validation in `list_all_changes_loaded`, workflow history) call `read_deadline::check(phase, done, total)` between items. When the deadline has passed, that returns `Err(ReadTimedOut { phase, progress })`, which is carried up as a typed error. The persistent read cache frame flushes what it holds on that error, as well as on success. The command layer turns it into exit 124 and the text or JSON report, with the resume command built from the command's own argv minus `--timeout`. Lifecycle writes never take a deadline.

**C. Seeded scopes for prefetch workers.** `prefetch_archived_workflow_histories` in `read_cache.rs` runs `validate_workflow_version_history_uncached` on up to 8 threads. The per-command repository-fact memo is thread-local, so those workers had no memo at all. Before spawning, the command takes `repository_fact_seed(root)`, a copy of what its open scopes for the root hold. Each worker opens `begin_seeded_repository_fact_scope(seed)`: the same fact rules, its own memo for its reads, and no batch reader. Workers only read, so the seeded facts stay true while they run. There is no seed while reads are live or a discovery ceiling is pinned, and then workers run as before.

## From the change's testing.md

# Testing

Gating, counted in Git processes, so independent of the machine:
- `warm_archived_status_git_processes_do_not_grow_with_legacy_changes` (new; fails now: 97 with 2 workflow-v1 changes, 115 with 8).
- A cold archived-status guard over archive-introducing commits (new; failing first).
- Equivalence: the one-walk index equals the per-commit index; workflow-v1 cutoff verdicts (absent, added after the cutoff, archived after it, a different id) equal today's.
- Timeout: `--timeout 0` on an archived status exits 124 with the JSON shape, every change file is byte-identical, and the verdicts completed so far are in the cache. The resume command gives the same verdict.
- The existing `fast_archived_reads` and `fast_finalize` guards, the hidden-downgrade tests and the full verify lane all pass unchanged.

On demand: `fledge run bench-archived-reads` with workflow-v1 changes in the fixture, against under 2 s warm and under 5 s cold on an unloaded Mac. This repository's archived status is timed before and after.

## Results

| Guard | Before (main) | After |
|---|---|---|
| `warm_archived_status_git_processes_do_not_grow_with_legacy_changes` (2 vs 8 archived workflow-v1 changes) | 94 vs 106, failed | 91 vs 91 |
| `cold_archive_introduction_index_does_not_grow_with_archive_introductions` (2 vs 6 archives) | 17 vs 21 with the one-walk listing disabled, failed | 16 vs 16 |
| `cold_archived_status_prefetch_workers_reuse_repository_facts` (2 vs 6 archives) | +250 | +135 (ceiling 150) |

On this repository (320 archives, debug build, load average 15-37), `change status <archived-id> --json`:

| Read | Before | After |
|---|---|---|
| Cold, Git processes | 14,482 | 5,580 |
| Cold, wall | 111 s | 30 s |
| Warm, Git processes | 262 | 111 |
| Warm, wall | 49-53 s | 5.9 s |

Every run reported `authenticated-history`. A cold read cut at `--timeout 3` exited 124 during `archived workflow history` after caching 2.1 MB of verdicts, and the printed resume command's JSON was identical to an uninterrupted run. The cold budget of under 5 s is not met; #915 tracks the gap with both options.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-135 | change tests `cold_archive_introduction_index_does_not_grow_with_archive_introductions` (fails with the one-walk listing disabled) and `one_walk_archive_introductions_equal_per_commit_introductions`; `an_unreadable_introduced_archive_blob_is_never_cached_as_absent` unchanged |
| REQ-change-136 | change tests `warm_archived_status_git_processes_do_not_grow_with_legacy_changes` (failed first, e843f156 and its fixture correction) and `cutoff_tree_verdict_equals_the_history_walk_verdict`; integration test `first_reachable_workflow_v1_state_requires_the_trusted_pre_v2_cutoff` unchanged |
| REQ-change-137 | integration test `an_archived_status_past_its_timeout_exits_124_and_names_how_to_resume` (byte-identical `.specsync/`, full verdict afterwards); manual resume on this repository recorded above |
| REQ-cmd-change-032 | integration test `an_archived_status_past_its_timeout_exits_124_and_names_how_to_resume` (`--timeout 0`, `--timeout=0`, `SPECSYNC_TIMEOUT=0`, JSON and text reports) |
| REQ-cli-args-024 | integration test `an_archived_status_past_its_timeout_exits_124_and_names_how_to_resume` (`--timeout -1` and `--timeout abc` refused with exit 2) |
| REQ-git-utils-011 | git_utils tests `a_seeded_worker_scope_starts_from_the_parents_facts` and `no_seed_without_a_scope_or_while_reads_are_live`; change test `cold_archived_status_prefetch_workers_reuse_repository_facts` |

## Where these lessons go

- `specs/change/change.3md`
- `specs/cmd_change/cmd_change.3md`
- `specs/cli_args/cli_args.3md`
- `specs/git_utils/git_utils.3md`
