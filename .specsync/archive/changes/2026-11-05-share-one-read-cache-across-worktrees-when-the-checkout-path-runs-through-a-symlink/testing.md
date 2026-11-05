---
change: share-one-read-cache-across-worktrees-when-the-checkout-path-runs-through-a-symlink
artifact: testing
---

# Testing

`cargo test --bin specsync -- a_second_worktree_reads_warm_from_the_shared_cache` failed on macOS before the fix and passes after it.
