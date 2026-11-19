---
change: prefetch-workers-inherit-the-cutoff-verdicts-the-command-thread-checked-and-persisted
artifact: context
---

# Context

#978, a Codex P2 on #918, deferred by Leif's call. In `prefetch_archived_workflow_histories`
(`src/change/read_cache.rs`) each worker opens only a seeded repository-fact scope. The cutoff
helpers reach the persistent read cache through a thread-local frame, so in a worker they neither
hit nor record. Each worker repeated the cutoff-tree read, and none persisted the verdict
REQ-change-136 keeps. Since #915/#979 the workers run only when the lifecycle-state history index
cannot answer.

Before spawning, the command thread now runs `validate_workflow_v2_baseline` for each pending
workflow-v1 archive. That is the same check a worker makes, and it persists each presence. The
repository-fact seed taken right after hands the workers its Git answers. A refusal is ignored
there, so the worker reports it with its own message.

Chough: spec-sync's `.chough/` is still 0.5.x until spec-sync#1073 merges; this change goes through
a Chough 0.6.0 session before review (Leif's rule, relayed by t370 on 2026-11-13).
