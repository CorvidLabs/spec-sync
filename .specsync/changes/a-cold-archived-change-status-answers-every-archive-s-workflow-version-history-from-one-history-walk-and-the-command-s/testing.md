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

## Requirement evidence

| Requirement | Evidence |
| --- | --- |
| REQ-change-140 | change tests `cold_archived_status_git_processes_do_not_grow_with_archives` (failing first, `0623412f`), the index equivalence tests and the fallback test (filled in at check time) |
| REQ-change-141 | the cold guard and the partial-clone test for workflow history (filled in at check time) |
