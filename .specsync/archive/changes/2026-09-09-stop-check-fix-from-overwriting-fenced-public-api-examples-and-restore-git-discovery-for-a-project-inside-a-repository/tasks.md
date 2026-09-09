---
change: stop-check-fix-from-overwriting-fenced-public-api-examples-and-restore-git-discovery-for-a-project-inside-a-repository
artifact: tasks
---

# Tasks

- [x] `fix_near_miss_headers` writes the original Public API slice; blanked copy is scan-only.
- [x] Integration test: near-miss heading plus fenced example preserves the fenced body.
- [x] `git_cmd` does not set `GIT_CEILING_DIRECTORIES`; nested subdirectory is a git repo.
- [x] Update mcp-security.md and CHANGELOG git-child bullets.
- [x] Synchronize `cmd_new` spec invariants, error cases, and consumes table.
- [x] Definition approved by 0xLeif; scoped check, review, and same-PR finalization follow on PR #770.
