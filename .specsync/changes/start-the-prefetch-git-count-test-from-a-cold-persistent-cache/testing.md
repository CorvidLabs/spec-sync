---
change: start-the-prefetch-git-count-test-from-a-cold-persistent-cache
artifact: testing
---

# Testing

`cargo test --bin specsync -- git_processes_spawned_by_prefetch_threads_are_counted` failed on the #842 tip (10 vs 86) and passes after the fix.
