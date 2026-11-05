# Lesson bundle: start-the-prefetch-git-count-test-from-a-cold-persistent-cache

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Start the prefetch Git-count test from a cold persistent cache
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change_tests/fast_archived_reads.rs
- **Acceptance**: git_processes_spawned_by_prefetch_threads_are_counted passes with finalize caching history verdicts, measuring the prefetch from a cold persistent cache.

## Evidence

- Verification commit: `de59550e60b174b4608271bf19962ef2d630627d`
- Base commit: `30be7ea14fb279875e39084f6624fb54c694c8a1`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

PR #842 CI failed `git_processes_spawned_by_prefetch_threads_are_counted` (added by #840): "prefetch reported 10 Git processes for work that spawns 86 serially". REQ-change-121 makes finalize store passing workflow-history verdicts in the persistent read cache, so the test's four setup finalizes left every archive cached and the prefetch skipped them. The production behavior is intended; the test assumed a cold cache.

## From the change's testing.md

# Testing

`cargo test --bin specsync -- git_processes_spawned_by_prefetch_threads_are_counted` failed on the #842 tip (10 vs 86) and passes after the fix.

## Where these lessons go

- `specs/change/change.3md`
