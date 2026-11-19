---
change: prefetch-workers-share-one-copy-of-the-command-s-seeded-repository-facts
artifact: context
---

# Context

#955, a Codex P2 on #918. `RepositoryFactSeed` held an owned map of `MemoizedGitOutput` values,
and `prefetch_archived_workflow_histories` cloned it once per worker, up to eight. A near-cap
64 MiB Git result could become more than 512 MiB of copies. Leif approved the fix on 2026-11-13
("#955 shared copy", coordinator chat 04:55Z, relayed by claude-code-t370).

The seed's facts now sit behind an `Arc`. A worker's frame reads them as `inherited`, after the
facts it learns itself, and invalidation clears both. The red test does not compile against main:
it asserts the clones share one allocation, and main's seed has no shared allocation to compare.

Chough: spec-sync's `.chough/` is still 0.5.x until spec-sync#1073 merges; this change goes through
a Chough 0.6.0 session before review (Leif's rule, relayed by t370 on 2026-11-13).
