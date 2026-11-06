---
change: a-cold-archived-change-status-answers-every-archive-s-workflow-version-history-from-one-history-walk-and-the-command-s
artifact: design
---

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
