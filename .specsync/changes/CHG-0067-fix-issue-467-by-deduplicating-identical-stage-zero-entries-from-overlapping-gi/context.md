---
change: CHG-0067-fix-issue-467-by-deduplicating-identical-stage-zero-entries-from-overlapping-gi
artifact: context
---

# Context

## Problem

`inspect_git_candidates` sends exact candidate pathspecs to Git in deterministic bounded batches.
When the candidate set includes both a directory and its exact tracked children, the directory can
return a child in one batch and the exact child pathspec can return the same stage-zero record in a
later batch. The current first-write-wins maps reject every second observation, even when both the
mode and object ID are identical.

## Decision

Accumulate each stage-zero path as one `(mode, object ID)` pair. An identical observation is an
idempotent duplicate and leaves the accumulated entry unchanged. Any observation whose mode or
object ID differs from the first pair fails closed with a deterministic conflicting-duplicate
error. Object IDs remain normalized to lowercase before comparison.

## Scope

The change is private to Git candidate inspection in `src/change.rs`. It does not change batching,
output limits, literal pathspec construction, unresolved-stage rejection, out-of-scope path
validation, dirty-worktree inspection, or the public Rust API.
