---
change: specsync-s-own-git-history-reads-ignore-git-replace-ref-base-and-git-no-replace-objects-as-they-already-ignore-git
artifact: design
---

# Design

**Recommended: clear the history-rewriting Git environment once, at process start.** At the top of `main`, before any thread starts, remove `GIT_REPLACE_REF_BASE`, `GIT_NO_REPLACE_OBJECTS`, `GIT_GRAFT_FILE` and `GIT_SHALLOW_FILE` from SpecSync's own environment. This is one `unsafe` block, sound because it runs single-threaded before anything reads the environment. Every Git process SpecSync spawns, configured, direct or through `git_cmd`, then walks the same history: the object graph plus `refs/replace/` and `info/grafts`. Those are exactly what the read-cache and index guards check, so the guards stay correct as written. `rooted_git_command` keeps its own list, which covers library callers that never go through `main`.

**Alternative for Leif (narrow):** add the two variables to `rooted_git_command`'s strip list only. That covers the configured path, but not the direct `Command::new("git")` spawns, which inherit them (and `GIT_GRAFT_FILE`) today.

Nothing else changes: the guards, the cache key, and every refusal.
