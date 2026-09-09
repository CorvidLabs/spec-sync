## ADDED

### REQUIREMENT REQ-git-utils-007

`git_cmd` SHALL be crate-visible so production git spawns outside this module can use the same cleared-environment allowlist.

Acceptance Criteria
- `git_cmd` is `pub(crate)`.
- Explicit env after `env_clear` does not include `GITHUB_TOKEN`, `AWS_SECRET_ACCESS_KEY`, `SPECSYNC_TEST_SECRET`, `GH_TOKEN`, `GIT_DIR`, `GIT_ASKPASS`, `GIT_INDEX_FILE`, or `SSH_AUTH_SOCK`.
- Default `git_cmd` remains ceiling-free; `with_discovery_ceiling` is still the only way a child receives `GIT_CEILING_DIRECTORIES`.
