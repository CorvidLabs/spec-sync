## ADDED

### REQUIREMENT REQ-git-utils-005

Every production `git` child spawned from `git_utils` SHALL start with a cleared environment, inherit only an allowlist of PATH/locale/home/temp variables, set `GIT_TERMINAL_PROMPT=0` and `GIT_OPTIONAL_LOCKS=0`, and pin `GIT_CEILING_DIRECTORIES` to the parent of the canonical project root.

Acceptance Criteria
- `GITHUB_TOKEN`, `GH_TOKEN`, `GIT_DIR`, `GIT_ASKPASS`, `GIT_INDEX_FILE`, and `SSH_AUTH_SOCK` are not on the allowlist.
- A directory that is not itself a repository, sitting inside a host worktree, is not reported as a git repo (walk-up is ceiling-stopped).
- A real repository root is still detected.
