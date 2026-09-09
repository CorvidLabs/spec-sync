---
change: isolate-mcp-snapshot-git-discovery-from-a-host-worktree-without-breaking-nested-project-walk-up
artifact: plan
---

# Plan

1. Add `with_discovery_ceiling` and thread-local wiring in `git_cmd`; pin with unit tests (nested walk-up, snapshot isolation, allowlist still denies ceiling inherit).
2. Wrap MCP snapshot tool and resource dispatch with ceiling=`parent(snapshot)`.
3. Pin MCP scoring: snapshot created inside a host worktree withholds git freshness once.
4. Update mcp-security.md and CHANGELOG.
5. Align `specs/cmd_new` companions with the already-synchronized spec body.
