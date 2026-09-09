---
change: stop-check-fix-from-overwriting-fenced-public-api-examples-and-restore-git-discovery-for-a-project-inside-a-repository
artifact: plan
---

# Plan

1. Write deltas for cmd_check, git_utils, cmd_new. Approve as 0xLeif with the overnight-brief note.
2. Implement header-rewrite buffer split, drop git ceiling, sync cmd_new spec, update mcp-security and CHANGELOG.
3. Pin with `fix_preserves_fenced_example_when_renaming_a_near_miss_header` and `is_git_repo_detects_project_inside_repository_subdirectory`.
4. `change check --commit`, pre-push, push, wait for CI on #770.
5. Reply to Codex threads; resolve the ones this package addresses.
6. `change review --reviewer 0xLeif` then `change ship` with no commit in between; commit archive tip; push.
