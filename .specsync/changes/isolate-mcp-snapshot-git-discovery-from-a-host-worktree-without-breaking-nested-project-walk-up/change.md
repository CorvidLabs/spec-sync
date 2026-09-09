---
id: isolate-mcp-snapshot-git-discovery-from-a-host-worktree-without-breaking-nested-project-walk-up
state: implementing
type: bug_fix
base_commit: 1f4f127f4741276a2e983b845be2d08a6a2641b4
---

# Isolate MCP snapshot git discovery from a host worktree without breaking nested-project walk-up

## Intent

isolate MCP snapshot git discovery from a host worktree without breaking nested-project walk-up

## Affected Canonical Specs

- `git_utils`
- `mcp`

## Acceptance Criteria

- MCP specsync_score on a snapshot whose tempfile sits inside a host git worktree reports git_freshness_available false, withholds git freshness once (no double penalty), and does not treat the host history as measurable; is_git_repo remains true for a nested project subdirectory when no snapshot isolation is in effect; git children still env_clear and drop GITHUB_TOKEN/GIT_DIR/GIT_CEILING_DIRECTORIES from the host environment.

## No-spec Rationale

Not applicable
