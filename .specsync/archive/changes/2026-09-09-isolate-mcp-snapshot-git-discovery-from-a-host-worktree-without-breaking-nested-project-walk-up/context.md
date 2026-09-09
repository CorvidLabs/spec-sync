---
change: isolate-mcp-snapshot-git-discovery-from-a-host-worktree-without-breaking-nested-project-walk-up
artifact: context
---

# Context

Codex review of PR #771 re-raised REQ-mcp-008 after #770's follow-up dropped `GIT_CEILING_DIRECTORIES` from every `git_cmd`.

Empirically (Git 2.39.5): `GIT_CEILING_DIRECTORIES=parent(project_root)` makes `git -C repo/sub` fail, which is why the follow-up removed the global pin so nested CLI projects still discover the parent work tree. The same topology with no ceiling makes a tempfile sitting inside a host worktree (`TMPDIR` pointing at the repo) discover that host. `ProjectSnapshot` always copies without `.git` and sets `git_freshness_available: false`, but `score_spec` still probes git on the snapshot path. Walk-up then marks freshness `Measured` (untracked mtime fallback) and `score_spec_for_mcp` deducts another 5 points — double penalty — while claiming git history is unavailable.

REQ-mcp-008 still requires: an MCP snapshot sitting inside a host worktree cannot walk up into that worktree. Nested-project walk-up remains the git_utils default. Isolation is snapshot-scoped, not global.

Also in this package (Codex P2 on the same PR): `specs/cmd_new` companions still name `chrono_lite_today` / `get_exported_symbols` after the spec body was synchronized. Companion-only; `src/commands/new.rs` unchanged.
