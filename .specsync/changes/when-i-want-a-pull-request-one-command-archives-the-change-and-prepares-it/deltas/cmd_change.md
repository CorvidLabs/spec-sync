## ADDED

### REQUIREMENT REQ-cmd-change-020

The command SHALL archive a ready change, push its branch, and prepare the pull request in one command. An existing pull request for that branch SHALL be left in place. The command SHALL NOT merge the pull request.

Acceptance Criteria
- `specsync change pr` archives a ready change that is still open, using the same commit as `change ship`.
- After the archive commit, the command pushes the current branch.
- When a pull request for the branch is already open, the command prints that pull request and does not open another.
- When no pull request is open, the command opens one.
- The command's next line says to wait until the checks are green and then merge.
- The command does not merge.
