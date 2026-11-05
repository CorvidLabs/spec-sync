---
change: finalize-a-change-in-seconds-on-raven-however-many-archives-it-holds
artifact: design
---

# Design

1. A `HistoryCacheScope` that `finalize_change` and `archive_change` open for their own duration. It shares the persistent, Git-keyed layer of REQ-change-119 (workflow-version history verdicts and archive introductions, under the same digest and history rule) and flushes it on drop. It takes no working-tree snapshot, so the command still sees its own writes.
2. Inside that scope, a memo for Git queries whose answer is a pure function of objects named by full id: the bytes of a path at a full commit id, `rev-list --parents -n 1 <id>`, `merge-base --is-ancestor <id> <id>`, plus the repository prefix and a positive repository detection. Queries naming `HEAD`, a branch, or the working tree are never memoized; failed process runs are not stored.
3. The record-independent half of the workflow-v2 baseline check moves into `validate_workflow_v2_baseline_history`, which names the resolved `HEAD` id in every query instead of `HEAD`. A passing verdict is memoized in the scope keyed by that id, the baseline path, the cutoff and the current baseline bytes; `HEAD` itself is still resolved live on every call, and refusals are always re-derived.
