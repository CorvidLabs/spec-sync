---
change: a-cold-archived-change-status-answers-every-archive-s-workflow-version-history-from-one-history-walk-and-the-command-s
artifact: testing
---

# Testing

Gating, counted in Git processes:
- `cold_archived_status_git_processes_do_not_grow_with_archives` (committed failing in `0623412f`: 99 with 2 archives, 234 with 6; must add fewer than 3 for 4 more archives).
- Equivalence: index-derived candidate paths, state-path history, parents and verdicts equal today's per-archive answers on the fixtures listed in plan.md step 2, including the same first refusal message.
- Fallback: with the index forced unavailable (over its bound), every verdict equals today's.
- Partial clone: a listed state blob that cannot be read still refuses and is not cached.
- Deadline: `an_archived_status_past_its_timeout_exits_124_and_names_how_to_resume` passes unchanged.
- Unchanged: the warm guards, `fast_archived_reads`, `fast_finalize`, the hidden-downgrade tests and the full verify lane.

On demand: `fledge run bench-archived-reads`, cold under 5 s and warm under 2 s on an unloaded Mac; this repository's cold and warm reads before and after.

## Results

| Guard | Before | After |
|---|---|---|
| `cold_archived_status_git_processes_do_not_grow_with_archives` (2 vs 6 archives) | 99 vs 234, failed (`0623412f`) | 91 vs 91 |
| `cold_archived_status_prefetch_workers_reuse_repository_facts` (fallback path, index off) | +135 (ceiling 150) | +145 (ceiling 150) |

`fledge run bench-archived-reads` (124 archives, 24 workflow-v1; load average 30-65, so not an unloaded Mac):

| Read | Before (#918) | After |
|---|---|---|
| Cold read scope | 34.1 s, 3,899 Git processes | 2.4 s, 90 |
| Warm read scope | 1.49 s, 91 | 1.84 s, 90 |
| `change list` | 1.0 s | 0.3 s, 10 |
| Unscoped (#921, out of scope) | 158 s | 216 s |

This repository (313 archives at `67aa7dc2`, release builds of #918 and of this change, load average 30-44), `change status <archived-id> --json`:

| Read | Before (#918) | After |
|---|---|---|
| Cold | 5,488 Git processes, 23.9 s | 111, 3.9 s |
| Warm | 111, 2.6 s | 110, 2.7 s |

Every run reported `authenticated-history`, and the before and after JSON are identical.

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-140 | change tests `cold_archived_status_git_processes_do_not_grow_with_archives` (failing first, `0623412f`), `lifecycle_index_answers_equal_per_change_walks_across_merges_and_refusals`, `lifecycle_index_answers_equal_per_change_walks_from_a_root_commit`, `lifecycle_index_answers_equal_per_change_walks_in_a_glob_named_subdirectory`, `lifecycle_index_applies_the_per_change_bounds_like_the_walks`, `lifecycle_history_parse_folds_merge_blocks_and_refuses_other_shapes`; fallback: every equivalence test runs each change with the index off too |
| REQ-change-141 | change tests `cold_archived_status_git_processes_do_not_grow_with_archives` and `an_unreadable_workflow_history_blob_refuses_with_and_without_the_index`; workers on the fallback path: `cold_archived_status_prefetch_workers_reuse_repository_facts`, `git_processes_spawned_by_prefetch_threads_are_counted`; deadline: `an_archived_status_past_its_timeout_exits_124_and_names_how_to_resume` |
