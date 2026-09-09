---
change: isolate-mcp-snapshot-git-discovery-from-a-host-worktree-without-breaking-nested-project-walk-up
artifact: requirements
---

# Requirements

Semantic deltas carry the SHALL statements:

- `REQ-git-utils-005` default git children still do not pin a ceiling; nested-project walk-up holds; host `GIT_CEILING_DIRECTORIES` is not inherited
- `REQ-git-utils-006` `with_discovery_ceiling` scopes `GIT_CEILING_DIRECTORIES` to git children spawned inside the closure
- `REQ-mcp-008` MCP snapshot dispatch uses that scope with ceiling=`parent(snapshot)` so a snapshot inside a host worktree cannot walk up
