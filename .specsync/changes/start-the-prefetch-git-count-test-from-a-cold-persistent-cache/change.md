---
id: start-the-prefetch-git-count-test-from-a-cold-persistent-cache
state: approved
type: bug_fix
base_commit: 30be7ea14fb279875e39084f6624fb54c694c8a1
---

# Start the prefetch Git-count test from a cold persistent cache

## Intent

Start the prefetch Git-count test from a cold persistent cache

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- git_processes_spawned_by_prefetch_threads_are_counted passes with finalize caching history verdicts, measuring the prefetch from a cold persistent cache.

## No-spec Rationale

Test-only fix: finalize now caches passing history verdicts (REQ-change-121), so the test clears the persistent cache before measuring; no contract text changes.
