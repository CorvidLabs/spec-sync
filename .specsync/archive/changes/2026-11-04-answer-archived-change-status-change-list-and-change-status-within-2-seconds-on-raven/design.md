---
change: answer-archived-change-status-change-list-and-change-status-within-2-seconds-on-raven
artifact: design
---

# Design

1. Lazy archived terminal evidence. Inside a read scope an archived record's verdict depends only on its own archived integrity, so it is computed for the asked-for record alone and memoized; the batch path still evaluates non-archived records against every record as successors.
2. Scope memos for `scoped_review_git_text`, `scoped_review_file_at_commit` and an archive index for `find_change_dir`, all bounded by `MAX_CHANGE_READ_CACHE_ENTRIES` and installed only by read scopes.
3. A persistent read cache at `git rev-parse --git-path specsync/read-cache-v1.json`. Workflow-version history verdicts are keyed by change id and validated by state digest, archive path, build identity (package version plus bundled validation limits), repository prefix, and one batched `git log --full-history -m --no-renames` from the recorded commit to `HEAD` over the change roots; the baseline half stays live. Archive introductions are keyed by commit and pathspec. Writes are atomic renames and failures are ignored.
4. Cold reads prefetch the uncached archived history verdicts on up to eight threads and memoize them; the archive walk still runs in order and returns the same first error.
