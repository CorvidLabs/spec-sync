---
change: reuse-one-cat-file-batch-reader-per-repository-root-across-nested-repository-fact-scopes
artifact: context
---

# Context

Codex flagged it on #869 (thread on `src/git_utils.rs`, `batched_object`), and it is filed as #891 item (1). Leif asked for it as its own small change (2026-11-05, relayed by orc t18).

`with_batch` looks only at the innermost frame for `root`. A lifecycle command opens a command-wide scope (`cmd_change`), then nests a project-lock scope (`acquire_project_lock`) and read scopes (`begin_change_read_scope`) for the same root. A nested scope that does a batched read starts its own `cat-file --batch` child, even when the outer scope already has one. That nested reader is also dropped when the nested scope closes, so the next outer read starts yet another. REQ-git-utils-010 allows at most one per root.

Constraints: invalidation must still close every frame's reader (a reader can hold a ref snapshot from before a write), and facts behave as before. Nothing outside `src/git_utils.rs` changes.
