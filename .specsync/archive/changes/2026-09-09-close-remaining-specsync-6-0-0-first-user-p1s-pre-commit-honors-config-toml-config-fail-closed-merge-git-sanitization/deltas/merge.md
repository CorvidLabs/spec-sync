## ADDED

### REQUIREMENT REQ-merge-003

`unmerged_paths` SHALL spawn git through `crate::git_utils::git_cmd`, so a `check` or MCP caller that only asks for unmerged paths does not forward host secrets or git-override variables.

Acceptance Criteria
- The git child starts with a cleared environment and the `git_utils` inherit allowlist; `GITHUB_TOKEN`, `AWS_SECRET_ACCESS_KEY`, and `SPECSYNC_TEST_SECRET` are absent from that child.
- `None` vs empty-set unknown-vs-clean contract is unchanged.
- `cached_unmerged_paths` inherits the sanitization because it calls `unmerged_paths`.
