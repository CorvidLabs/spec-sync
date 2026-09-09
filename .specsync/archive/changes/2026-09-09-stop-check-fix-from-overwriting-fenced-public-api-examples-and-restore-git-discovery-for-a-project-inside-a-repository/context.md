---
change: stop-check-fix-from-overwriting-fenced-public-api-examples-and-restore-git-discovery-for-a-project-inside-a-repository
artifact: context
---

# Context

Codex review of PR #770 found two first-user P1s introduced by the overnight package, plus stale `cmd_new` spec prose.

1. `fix_near_miss_headers` scans a `blank_fenced_code` copy (correct) then `replace_range`s that blanked copy back into the spec (wrong). A Public API section that has both a fenced example and a near-miss or bare `###` heading has its fenced body overwritten with spaces. REQ-cmd-check-016 already required the fenced sample to be preserved verbatim.
2. `git_cmd` sets `GIT_CEILING_DIRECTORIES` to `parent(root)`. Empirically that makes `git -C repo/sub` fail to discover `repo/.git`. Nested-project layout is the git_utils contract (`is_git_repo` = inside a work tree; `source_was_deleted` resolves relative to a subdirectory root). The overnight MCP P1 was env sanitization (`GITHUB_TOKEN` / `GIT_DIR`); the ceiling over-reached.

Out of scope (Codex P2s, replied on the PR): generator Created-changelog row, extra `required_sections` headings, pre-upgrade CRLF approval digests, `include_str!` of the CI copy from `change_tests`.
