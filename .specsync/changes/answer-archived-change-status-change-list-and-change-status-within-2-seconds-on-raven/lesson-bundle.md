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

- Verification commit: `b09a82cd6fe3fd8fd3031177a50f4349982a4ae2`
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

REQ-change-119 is covered by archived_status_git_processes_do_not_grow_with_other_archives (failed first on main: three more archived changes added 237 Git processes to one read, 210 with two archives and 447 with five), cached_archived_reads_answer_exactly_like_live_reads, cached_archived_history_still_refuses_a_later_hidden_downgrade, cached_archived_reads_recheck_an_edited_archive_and_a_moved_head and path_is_state_candidate_matches_only_one_change_s_state_paths. raven_sized_archived_status_benchmark is the timed benchmark (`fledge run bench-archived-reads`). Raven timings before and after are recorded in the pull request.

## Where these lessons go

- `specs/change/change.3md`
