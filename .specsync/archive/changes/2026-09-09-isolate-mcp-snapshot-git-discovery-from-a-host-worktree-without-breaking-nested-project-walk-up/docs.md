---
change: isolate-mcp-snapshot-git-discovery-from-a-host-worktree-without-breaking-nested-project-walk-up
artifact: docs
---

# Docs

- `site/src/content/docs/mcp-security.md`: CLI git children still do not pin `GIT_CEILING_DIRECTORIES`. MCP snapshot tool/resource dispatch pins the ceiling to the snapshot parent so a tempfile inside a host worktree cannot walk up. Env sanitization unchanged.
- CHANGELOG `[6.0.0]` Fixed: MCP snapshots inside a host worktree no longer discover host git history.
