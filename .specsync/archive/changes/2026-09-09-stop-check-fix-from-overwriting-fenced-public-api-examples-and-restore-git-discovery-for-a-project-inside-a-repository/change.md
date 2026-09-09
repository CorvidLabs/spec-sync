---
id: stop-check-fix-from-overwriting-fenced-public-api-examples-and-restore-git-discovery-for-a-project-inside-a-repository
state: archived
type: bug_fix
base_commit: cd812fd2d0b70365f93555841c665849d0a48265
---

# Stop check --fix from overwriting fenced Public API examples and restore git discovery for a project inside a repository subdirectory

## Intent

stop check --fix from overwriting fenced Public API examples and restore git discovery for a project inside a repository subdirectory

## Affected Canonical Specs

- `cmd_check`
- `git_utils`
- `cmd_new`

## Acceptance Criteria

- check --fix with a near-miss Public API heading plus a fenced example preserves the fenced body verbatim; is_git_repo is true for a project subdirectory of a git repository; git children still env_clear and drop GITHUB_TOKEN/GIT_DIR; cmd_new spec describes generate_spec/validate_scaffold_module_name rather than chrono_lite_today.

## No-spec Rationale

Not applicable
