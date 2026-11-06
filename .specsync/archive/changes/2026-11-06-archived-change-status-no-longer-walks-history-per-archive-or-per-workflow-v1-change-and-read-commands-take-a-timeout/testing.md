---
change: archived-change-status-no-longer-walks-history-per-archive-or-per-workflow-v1-change-and-read-commands-take-a-timeout
artifact: testing
---

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
