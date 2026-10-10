## MODIFIED

### REQUIREMENT REQ-cmd-change-022
`change finalize` SHALL commit the archive move when finalization succeeds. The commit subject SHALL be `chore(lifecycle): archive <id>`. The active change directory SHALL NOT remain an uncommitted rename. When the package is already archived and the commit is still missing, running finalize again SHALL commit that package and SHALL NOT fail because the state is archived. The next action SHALL tell the operator to push the archive commit before merging.

Acceptance Criteria
- After `specsync change finalize <id>` succeeds, `git log -1` contains `chore(lifecycle): archive <id>`.
- `git status --porcelain` does not show `.specsync/changes/<id>`.
- Finalize after an uncommitted archive move still creates that commit.
- The next action says to push the archive before merging.
- The command does not merge the pull request.
