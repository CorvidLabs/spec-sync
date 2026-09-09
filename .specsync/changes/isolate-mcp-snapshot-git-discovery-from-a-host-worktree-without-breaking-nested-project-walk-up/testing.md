---
change: isolate-mcp-snapshot-git-discovery-from-a-host-worktree-without-breaking-nested-project-walk-up
artifact: testing
---

# Testing

| ID | Evidence |
|----|----------|
| REQ-git-utils-005 | `git_inherited_env_excludes_secrets_and_git_overrides` (deny list includes `GIT_CEILING_DIRECTORIES`), `is_git_repo_detects_project_inside_repository_subdirectory` |
| REQ-git-utils-006 | `discovery_ceiling_isolates_a_subdirectory_from_host_walk_up`, `discovery_ceiling_does_not_leak_after_the_closure` |
| REQ-mcp-008 | `mcp_snapshot_inside_host_worktree_does_not_discover_host_git_or_double_penalize` |
