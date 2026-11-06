# Lesson bundle: a-cold-archived-change-status-answers-every-archive-s-workflow-version-history-from-one-history-walk-and-the-command-s

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: A cold archived change status answers every archive's workflow-version history from one history walk and the command's batch reader
- **Kind**: Feature
- **Specs**: change
- **Paths**: src/change/validate_loaded_change.rs, src/change/read_cache.rs, src/change/historical_definition_digest_matches.rs, src/change_tests/fast_archived_reads.rs, specs/change, CHANGELOG.md, hi/speed.md, INTENT.md
- **Acceptance**: A cold change status of an archived change spawns fewer than 3 more Git processes when the project holds 4 more archives (cold_archived_status_git_processes_do_not_grow_with_archives, committed failing first in 0623412f: 99 with 2 archives, 234 with 6)
- **Acceptance**: Inside a change read scope, every archived change's workflow-version history is answered from one lifecycle-state history walk per command: candidate state paths, oldest-first state-path history and each commit's parents are derived in memory instead of four Git walks per archive
- **Acceptance**: The state.json reads and path-exists checks behind that validation go through the command's one object batch reader on the command's thread; a listed blob that cannot be read still refuses and is never cached as absent
- **Acceptance**: Every archive is still fully validated, so authenticated-history keeps its meaning: equivalence tests show the one-walk verdict equals the per-archive verdict, including the same first refusal and message, for merges, a moved or re-archived package, a deleted creation anchor, a changed identity, a non-canonical archive path and the per-archive history bound
- **Acceptance**: When the walk cannot answer (over its bound, over the output limit, or a failed Git call) each archive falls back to today's per-archive check, so no new refusal is introduced
- **Acceptance**: The read deadline (REQ-change-137) still stops the validation between archives with phase and progress, and a warm archived status still spawns no more processes than today
- **Acceptance**: On demand (fledge run bench-archived-reads) a cold archived status is under 5 s on an unloaded Mac and warm stays under 2 s; cold and warm times on this repository are recorded before and after
- **Acceptance**: No gate, bound, timeout default or approval rule is weakened, and no existing assertion is loosened

## Evidence

- Verification commit: `522e629edb0fe90f57e741616f4cda678dad56f0`
- Base commit: `0623412fe1e3cd6e1cc636efb097d100d468a631`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Issue #915, the follow-up S9 (#918) named. #918 brought a cold `change status <archived-id>` from 14,482 Git processes to about 5,600 on this repository, but it still grows with the archive count: 34.1 s and 3,899 processes on the Raven-sized benchmark, and 99 then 234 processes in the guard fixture for 2 then 6 archives. The cold under-5 s budget from S9's approval was measured and reported there, not required.

Leif chose option 3 of the #915 proposal and approved SPEED-1.a.1 as drafted on the morning of 2026-11-06 ("Ok yes I approve all"), relayed by orc t18. Option 3 is one history walk for all archives plus the command's single batch reader, with every archive still fully validated. Option 2 (validate only the target archive) was not chosen, so `authenticated-history` keeps its meaning. The orc reserved REQ-change-140 and REQ-change-141 for this change.

This branch is stacked on #918 at `1737406b`. The failing guard is `0623412f`. When #918 merges, origin/main is merged in here before the PR.

The unscoped read (#921, `change correct`) is not in scope. It opens no read scope, so it gets none of this.

## From the change's design.md

# Design

**One walk per command.** A per-command lifecycle-state history index, held in the change read scope for the root and keyed by the `HEAD` object id, is built the first time an archived change's workflow-version history is needed. It costs a fixed number of Git processes, whatever the archive count:

1. One `git log --full-history --name-only -z` from `HEAD` over `:(top,glob)<changes root>/*/state.json` and `:(top,glob)<archive root>/*/state.json`, bounded by `MAX_TRUSTED_HISTORY_COMMITS`. It lists every commit that touched a lifecycle state path, in the walk's own order.
2. Each listed commit's real parents, read from its commit object through the batch reader (or one `rev-list --no-walk --parents --stdin`), never from a parent-rewritten walk.
3. The paths each commit changed against each of its parents, from one `diff-tree --stdin -r --name-only` fed `<commit> <parent>` pairs, so a merge's changes are attributed to the right parent.

From the index, `validate_workflow_version_history_uncached` and `workflow_version_state_paths` derive, per archive, what they ask Git today:
- **Candidate state paths.** The archive paths that match `<archive root>/*-<id>/state.json` with `*` not crossing `/`, as `:(glob)` matches them. Every candidate still goes through `is_canonical_archive_state_path`, so a non-canonical path refuses with the same message.
- **State-path history, oldest first.** The commits with at least one parent whose change list touches one of the archive's state paths (a root commit counts against the empty tree). That is the rule `rev-list --full-history` without parent rewriting applies to a path-limited walk. The walk visits every commit reachable from `HEAD`, so filtering it keeps the per-archive walk's order. The per-archive `scoped_review_max_descendants` bound and its refusal apply to that filtered list, as today.
- **Parents.** From step 2, with the same `scoped_review_max_parents` refusal.

**Reads through the batch reader on the command thread.** When the index answers, `prefetch_archived_workflow_histories` validates the pending archives on the command's own thread, not on workers. `scoped_review_file_at_commit` and `commit_tree_lists_path` are then served by the command's one `cat-file --batch` child (REQ-git-utils-010), with the same object-and-tree semantics #869 pinned: a missing entry is absent, and a listed blob that cannot be read is an error, never cached as absent. With about a dozen batch reads per archive, 320 archives cost a few thousand pipe round trips and no extra processes. Workers, and REQ-git-utils-011, stay as the fallback.

**Fallback, never a new refusal.** If the index cannot be built (over the commit bound or the output limit, a failed Git call, non-UTF-8 output, or no read scope), each archive runs today's per-archive check unchanged. The index is an answer source, not a judge, so it can make a check faster but never make it refuse differently.

**Deadline.** The serial loop calls `check_read_deadline("archived workflow history", done, total)` between archives (REQ-change-137), so a timed-out read still reports progress and flushes completed verdicts.

**Not persisted.** The walk takes 0.05 s here, and a warm read already skips it through `cached_workflow_history_is_current`. Nothing new goes into the read cache file, so its format and key stay as they are.

## From the change's testing.md

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

## Where these lessons go

- `specs/change/change.3md`
