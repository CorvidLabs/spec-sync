---
change: a-cold-archived-change-status-answers-every-archive-s-workflow-version-history-from-one-history-walk-and-the-command-s
artifact: plan
---

# Plan

## Approach

Each archive's workflow-version history check asks four history questions of Git, plus one process per blob and path check. A cold read therefore grows with the archive count. Replace the per-archive history questions with one per-command lifecycle-state history index, and run the blob and path reads through the command's batch reader on its own thread (design.md). Every archive is still fully validated, with the same verdicts and refusals, and the per-archive check stays as the fallback.

## Out of scope

- Option 2 (validating only the target archive) and any change to what `authenticated-history` means.
- The unscoped read (#921, `change correct`), and #887 (memoizing nonzero exits).
- Persisting the index in the read cache, and any change to the read-cache format or key (#920).
- Lifecycle writes, finalize's structural roster, and `change audit --archives`.

## Steps

1. The failing guard `cold_archived_status_git_processes_do_not_grow_with_archives` is already committed (`0623412f`: 99 with 2 archives, 234 with 6).
2. Equivalence tests first, comparing index-derived answers with today's per-archive Git answers on fixtures with: a merge that touches a state path from either parent, a package moved or re-archived under a new date, an active path and an archive path for one id, a root commit, a project in a subdirectory (#851), a non-canonical archive directory, a deleted creation anchor, a changed workflow identity, and a history over the per-archive bound.
3. The lifecycle-state history index (walk, parents, per-parent changes) and its fallback rules.
4. `workflow_version_state_paths` and `validate_workflow_version_history_uncached` read from the index when it answers.
5. `prefetch_archived_workflow_histories` validates on the command thread through the batch reader when the index answers; workers remain the fallback. Deadline checks between archives.
6. The cold guard turns green; the warm guards, `fast_archived_reads`, `fast_finalize`, the hidden-downgrade tests and the timeout test pass unchanged.
7. `fledge run bench-archived-reads` cold and warm, and this repository's cold and warm reads, recorded before and after.
8. Spec rows and the `change` lesson, CHANGELOG; `change check --commit`; the verify lane.

## Risks

- **Path-limited history simplification.** A walk over all state paths and a walk over one archive's paths must select the same commits. The per-parent change lists apply the `--full-history` rule exactly, and the equivalence tests pin it, including merges. If a history shape does not match, that archive falls back to today's check.
- **Merge attribution.** `log -m` does not say which parent a block is against, so per-parent changes come from `diff-tree --stdin` with explicit pairs.
- **Paths reported from another root** (#879). Walk output is repository-relative; the subdirectory-project fixture covers it.
- **Partial clones.** A listed blob that cannot be read must still refuse. The existing `an_unreadable_introduced_archive_blob_is_never_cached_as_absent`-style test is extended to workflow history.
- **The guard may show growth outside workflow history.** If it does, I report the remaining shape rather than loosen the guard.
- **Timings swing with load** on this shared Mac. The gating test counts processes; times come from the on-demand benchmark.

## Constraints consulted

- SPEED-1.a, SPEED-1.a.1 and SPEED-1.d.
- change.3md: "Finalize judges only what it archives" (read paths keep live-identical refusals), "History pathspecs from a subdirectory" (#851), "Working-tree paths from a subdirectory" (#879).
- REQ-change-135, 136 and 137, and REQ-git-utils-010 and 011, unchanged.
- The partial-clone rule: an unreadable listed blob is never absent.
