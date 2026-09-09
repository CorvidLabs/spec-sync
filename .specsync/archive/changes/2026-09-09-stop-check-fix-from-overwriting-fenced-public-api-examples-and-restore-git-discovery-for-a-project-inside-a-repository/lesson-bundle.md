# Lesson bundle — stop-check-fix-from-overwriting-fenced-public-api-examples-and-restore-git-discovery-for-a-project-inside-a-repository

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Stop check --fix from overwriting fenced Public API examples and restore git discovery for a project inside a repository subdirectory
- **Kind**: BugFix
- **Specs**: cmd_check, git_utils, cmd_new
- **Paths**: src/commands/check.rs, src/git_utils.rs, tests/integration/fix.rs, site/src/content/docs/mcp-security.md, CHANGELOG.md
- **Acceptance**: check --fix with a near-miss Public API heading plus a fenced example preserves the fenced body verbatim; is_git_repo is true for a project subdirectory of a git repository; git children still env_clear and drop GITHUB_TOKEN/GIT_DIR; cmd_new spec describes generate_spec/validate_scaffold_module_name rather than chrono_lite_today.

## Evidence

- Verification commit: `6795224b84534d96087c6fb9bed2fb27d3f0b196`
- Base commit: `cd812fd2d0b70365f93555841c665849d0a48265`
- Verified by: `specsync check --spec cmd_check --spec cmd_new --spec git_utils`

## From the change's context.md

# Context

Codex review of PR #770 found two first-user P1s introduced by the overnight package, plus stale `cmd_new` spec prose.

1. `fix_near_miss_headers` scans a `blank_fenced_code` copy (correct) then `replace_range`s that blanked copy back into the spec (wrong). A Public API section that has both a fenced example and a near-miss or bare `###` heading has its fenced body overwritten with spaces. REQ-cmd-check-016 already required the fenced sample to be preserved verbatim.
2. `git_cmd` sets `GIT_CEILING_DIRECTORIES` to `parent(root)`. Empirically that makes `git -C repo/sub` fail to discover `repo/.git`. Nested-project layout is the git_utils contract (`is_git_repo` = inside a work tree; `source_was_deleted` resolves relative to a subdirectory root). The overnight MCP P1 was env sanitization (`GITHUB_TOKEN` / `GIT_DIR`); the ceiling over-reached.

Out of scope (Codex P2s, replied on the PR): generator Created-changelog row, extra `required_sections` headings, pre-upgrade CRLF approval digests, `include_str!` of the CI copy from `change_tests`.

## From the change's design.md

# Design

- `fix_near_miss_headers` keeps the original Public API slice as the replacement buffer. `blank_fenced_code` is scan-only (header discovery). Replacements apply by blanked-copy offsets into the original, end-to-start, so a length-changing rename (`### Functions` → `### Exported Functions`) does not shift later matches. A fenced `### Exportd Functions` stays quoted text because it is spaces in the scan copy.
- `git_cmd` keeps `env_clear` + the PATH/locale/home/temp allowlist + `GIT_TERMINAL_PROMPT=0` / `GIT_OPTIONAL_LOCKS=0` / `LC_ALL=C`. It does **not** set `GIT_CEILING_DIRECTORIES`. Inherited ceiling is already wiped by `env_clear`. Nested-project walk-up is required. Snapshot isolation is the caller's job (place the snapshot outside any repo).
- `cmd_new` spec body is synchronized to the implementation already on this PR: `generate_spec`, `validate_scaffold_module_name`, `check_case_collision`, `collect_exports_for_files` / `generate_companion_files_for_spec`. No `src/commands/new.rs` change.

## From the change's testing.md

# Testing

| ID | Evidence |
|----|----------|
| REQ-cmd-check-016 | `fix_preserves_fenced_example_when_renaming_a_near_miss_header` (plus existing `fix_ignores_fenced_exported_heading_before_the_table`, `fix_near_miss_handles_levenshtein_typos`) |
| REQ-git-utils-005 | `git_inherited_env_excludes_secrets_and_git_overrides`, `is_git_repo_detects_project_inside_repository_subdirectory` |
| REQ-cmd-new-001 | existing `new_auto_detects_single_source_file`; spec prose sync only |

## Where these lessons go

- `specs/cmd_check/context.md`
- `specs/git_utils/context.md`
- `specs/cmd_new/context.md`
