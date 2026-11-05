---
change: start-the-prefetch-git-count-test-from-a-cold-persistent-cache
artifact: context
---

# Context

PR #842 CI failed `git_processes_spawned_by_prefetch_threads_are_counted` (added by #840): "prefetch reported 10 Git processes for work that spawns 86 serially". REQ-change-121 makes finalize store passing workflow-history verdicts in the persistent read cache, so the test's four setup finalizes left every archive cached and the prefetch skipped them. The production behavior is intended; the test assumed a cold cache.
