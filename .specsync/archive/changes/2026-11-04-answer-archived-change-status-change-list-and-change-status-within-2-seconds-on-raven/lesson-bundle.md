# Lesson bundle: answer-archived-change-status-change-list-and-change-status-within-2-seconds-on-raven

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Answer archived change status, change list and change status within 2 seconds on Raven
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change.rs, src/change/consider_accepted_evidence_anchor.rs, src/change/historical_definition_digest_matches.rs, src/change/sdd_version.rs, src/change/validate_loaded_change.rs, src/change/read_cache.rs, src/change_tests.rs, src/change_tests/fast_archived_reads.rs, specs/change, hi/speed.md, INTENT.md, fledge.toml
- **Acceptance**: Archived change status, change list and change status return in under 2 seconds on Raven, check stays proportional to the specs touched, read caches are keyed by content digest with a correct invalidation rule, and a benchmark against a Raven-sized fixture guards the budget; output is identical to a live read.

## Evidence

- Verification commit: `cd2853b1813fd12e14e9fb72c3bffdc584dcbf1f`
- Base commit: `84dbb56b245da36b3625440f5e394d85d686cd22`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Leif asked in Grok Bot chat t811u (2026-11-03) for status of an archived change, `change list` and `change status` to return in under 2 seconds on Raven, with `check` staying proportional to the specs touched, caching by content digest with a correct invalidation rule, and a benchmark against a Raven-sized fixture (hi SPEED-1). On Raven (124 archived changes, 120 specs) 6.1.1 took more than 2 minutes on Apple silicon for one archived status read.

Profiled on Linux against CorvidLabs/raven main 74e43005: one archived `change status <id> --json` took 40.4 s with 6.1.1 and 40.8 s with main, spawning 9,972 Git processes. Three costs dominated: terminal evidence was computed for every accepted or archived record when only one was asked for; scoped-review Git reads were repeated without a memo; and every archive's workflow-version history and archive introduction were re-walked on every call. Active-change reads were 0.13 to 1.7 s and `check` 1.6 to 3.5 s on Linux, so they were already proportional; the minutes Leif saw there are consistent with the archived reads underneath plus macOS process cost.

## From the change's design.md

# Design

1. Lazy archived terminal evidence. Inside a read scope an archived record's verdict depends only on its own archived integrity, so it is computed for the asked-for record alone and memoized; the batch path still evaluates non-archived records against every record as successors.
2. Scope memos for `scoped_review_git_text`, `scoped_review_file_at_commit` and an archive index for `find_change_dir`, all bounded by `MAX_CHANGE_READ_CACHE_ENTRIES` and installed only by read scopes.
3. A persistent read cache at `git rev-parse --git-path specsync/read-cache-v1.json`. Workflow-version history verdicts are keyed by change id and validated by state digest, archive path, build identity (package version plus bundled validation limits), repository prefix, and one batched `git log --full-history -m --no-renames` from the recorded commit to `HEAD` over the change roots; the baseline half stays live. Archive introductions are keyed by commit and pathspec. Writes are atomic renames and failures are ignored.
4. Cold reads prefetch the uncached archived history verdicts on up to eight threads and memoize them; the archive walk still runs in order and returns the same first error.

## From the change's testing.md

# Testing

REQ-change-119 is covered by archived_status_git_processes_do_not_grow_with_other_archives (failed first on main: three more archived changes added 237 Git processes to one read, 210 with two archives and 447 with five), cached_archived_reads_answer_exactly_like_live_reads, cached_archived_history_still_refuses_a_later_hidden_downgrade, cached_archived_reads_recheck_an_edited_archive_and_a_moved_head and path_is_state_candidate_matches_only_one_change_s_state_paths. Review follow-ups, each committed failing first: cached_archived_history_refuses_a_hidden_downgrade_in_a_renamed_archive (renamed archive bypass), an_unreadable_introduced_archive_blob_is_never_cached_as_absent and an_unreadable_blob_at_a_commit_is_an_error_not_an_absent_file (offline partial clone), a_pass_is_never_stamped_with_a_head_that_moved_during_the_read, a_cache_hit_moves_its_entry_forward_to_the_current_head, a_cache_flush_never_shares_its_temporary_file_with_another_writer, git_processes_spawned_by_prefetch_threads_are_counted, the_persistent_cache_is_off_while_replace_refs_rewrite_history, the_persistent_cache_is_off_while_grafts_rewrite_history, the_persistent_cache_is_off_in_a_partial_clone, a_second_worktree_reads_warm_from_the_shared_cache and a_pass_is_stored_only_for_the_state_it_was_computed_for. raven_sized_archived_status_benchmark is the timed benchmark (`fledge run bench-archived-reads`). Raven timings before and after are recorded in the pull request.

## Where these lessons go

- `specs/change/change.3md`
