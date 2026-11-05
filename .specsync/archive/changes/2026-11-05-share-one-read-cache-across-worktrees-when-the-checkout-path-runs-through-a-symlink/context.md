---
change: share-one-read-cache-across-worktrees-when-the-checkout-path-runs-through-a-symlink
artifact: context
---

# Context

`a_second_worktree_reads_warm_from_the_shared_cache` (#840) fails on macOS, where the release candidate is qualified: the main worktree's `git rev-parse --git-common-dir` is the relative `.git`, joined onto the root as given (`/var/...`), while a linked worktree gets Git's absolute canonical path (`/private/var/...`). The two worktrees then use different cache files. CI's product lane is Ubuntu-only, so it never saw this.
