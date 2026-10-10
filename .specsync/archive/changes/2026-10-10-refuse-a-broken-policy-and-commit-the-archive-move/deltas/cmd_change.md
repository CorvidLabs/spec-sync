## ADDED

### REQUIREMENT REQ-cmd-change-021
`change status` SHALL fail when `.specsync/sdd.json` cannot be parsed. The error SHALL be the invalid SDD policy error. Status SHALL NOT print a briefing as if no principles file were configured.

Acceptance Criteria
- Status returns the error from `configured_principles_file` before it prints a briefing.
- The error contains `invalid SDD policy`.
- An unreadable principles file, when the policy itself parses, still does not fail status.

### REQUIREMENT REQ-cmd-change-022
`change finalize` SHALL commit the archive move when finalization succeeds. The commit subject SHALL be `chore(lifecycle): archive <id>`. The active change directory SHALL NOT remain an uncommitted rename.

Acceptance Criteria
- After `specsync change finalize <id>` succeeds, `git log -1` contains `chore(lifecycle): archive <id>`.
- `git status --porcelain` does not show `.specsync/changes/<id>`.
- The command does not merge the pull request.
