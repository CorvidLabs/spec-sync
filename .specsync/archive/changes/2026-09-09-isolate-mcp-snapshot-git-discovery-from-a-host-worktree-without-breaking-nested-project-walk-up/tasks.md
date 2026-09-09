---
change: isolate-mcp-snapshot-git-discovery-from-a-host-worktree-without-breaking-nested-project-walk-up
artifact: tasks
---

# Tasks

- [x] `with_discovery_ceiling` + `git_cmd` wiring; default path still ceiling-free
- [x] Unit tests: nested subdirectory stays a git repo; snapshot-in-host with ceiling=`parent(snapshot)` does not; allowlist still omits `GIT_CEILING_DIRECTORIES`
- [x] MCP tool/resource snapshot dispatch wraps git probes with that ceiling
- [x] MCP score test: tempfile inside a host worktree withholds git freshness once
- [x] mcp-security.md + CHANGELOG
- [x] `specs/cmd_new` companions drop `chrono_lite_today` / `get_exported_symbols`
- [x] Definition approved by 0xLeif; scoped check, review, and same-PR finalization follow on PR #771
