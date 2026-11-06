---
change: reuse-one-cat-file-batch-reader-per-repository-root-across-nested-repository-fact-scopes
artifact: tasks
---

# Tasks

- [x] Failing unit test: a nested same-root scope reuses the outer reader, and an outer read after a nested-scope read starts no new child
- [x] `with_batch` searches every same-root frame, outermost first, for an open reader before starting one; a new reader goes in the outermost same-root frame
- [x] Invalidation still closes every reader (existing tests)
- [x] `lifecycle_commands_stay_within_git_spawn_ceilings` passes with no count rising
- [x] Spec: REQ-git-utils-010 acceptance criterion, invariant and Change Log row
