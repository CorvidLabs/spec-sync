---
change: restore-archived-history-reads-for-a-project-in-a-repository-subdirectory
artifact: context
---

# Context

Issue #851, found while reviewing #840. It is also on main. History queries build a repository-relative path with `git_repo_prefix`, then run Git with the project root as the working directory. Git resolves a pathspec from that directory unless the pathspec carries `:(top)`. When the prefix is non-empty, the prefix is applied twice and the query matches nothing.

Two reads then agree on an empty archived history. `workflow_version_state_paths` never sees earlier dated archive paths, and the walk of the paths it does return never sees the commits that touched them. `lifecycle_paths_touched_since` never sees a later edit, so a cached passing verdict stays current. A workflow-version downgrade that a later commit restored is invisible, live and from the cache.

`archive_introduction_index` already anchors the same kind of path with `:(top,literal)`. `git show <commit>:<path>` is already relative to the repository root and is left alone. The fix is the same anchor: `:(top,glob)` for the dated-archive discovery glob, `:(top,literal)` for the history walk of those paths and for the cache invalidation walk.

A fixture whose project is `app/` inside the repository is what makes the double prefix visible. The same checks at the repository root stay green either way.
