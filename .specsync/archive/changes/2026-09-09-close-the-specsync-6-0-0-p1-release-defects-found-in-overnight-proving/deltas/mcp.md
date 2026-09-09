## ADDED

### REQUIREMENT REQ-mcp-008

MCP scoring and any other MCP path that probes git history SHALL spawn `git` through `git_utils`, inheriting the sanitized child environment rather than the parent process environment.

Acceptance Criteria
- A read-only `specsync mcp` process that holds `GITHUB_TOKEN` for issue verification does not forward that token to `git`.
- An MCP snapshot sitting inside a host worktree cannot walk up into that worktree via `GIT_DIR` walk-up.
- Test fixtures constructing `GenerationOutcome` populate `skipped_no_files`.
