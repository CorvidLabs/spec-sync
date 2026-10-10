## MODIFIED

### REQUIREMENT REQ-cmd-change-019
The command SHALL make a pull request show whether its change is still open or already archived. An open change SHALL leave the pull request unfinished. When every change is archived, the pull request SHALL say it is ready to merge once the other checks are green. When it is time to archive, `change ship` SHALL archive the change, commit that archive, and say that merging is the step that is left. When it is not time, the same place SHALL say the one reason and the one command. When more than one change is open, the pull request SHALL say which one to archive first.

Acceptance Criteria
- The Archive check fails while a change workspace is open, names the one to archive first, and names one command.
- The Archive check passes only when no change workspace is open, and its message says the pull request is ready to merge when the other checks are green.
- The required CI gate fails while the Archive check fails.
- SpecSync implementation ready does not fail only because the Archive check failed.
- `change ship` on a ready change commits the archive tip and its next line says to push, wait until the checks are green, and then merge.
- `change ship` on a change that is not ready names one reason and one command and does not say to merge.
- When several changes are open, the one closest to archive is named first. Accepted comes before verifying, and a tie goes to the lowest id.
- A workspace whose state says archived but that still sits in `.specsync/changes/` is still open.
- `change ship` recommends `change ship` only for a workflow-v2 verifying change whose review verdict is pass and whose reviewed commit is HEAD. A block, a legacy record, or a review of another commit is told to run `change status`.

### REQUIREMENT REQ-cmd-change-020
The command SHALL archive a ready change, push its branch, and prepare the pull request in one command. An existing pull request for that branch SHALL be left in place. The command SHALL NOT merge the pull request.

Acceptance Criteria
- `specsync change pr` archives a ready change that is still open and is the one to archive first, using the same commit as `change ship`.
- A change that is not ready is not archived. The command still pushes the branch and opens or reuses the pull request, then names the one reason and the one command.
- The command runs `fledge lanes run pre-push` before it pushes. A failed gate does not push.
- After the archive commit, the command pushes the current branch.
- When a pull request for the branch is already open, the command prints that pull request and does not open another.
- A closed or merged pull request is not reused.
- When no pull request is open, the command opens one. When `origin/HEAD` cannot be read, the base branch is left for GitHub to choose.
- When every change is archived, the next line says to wait until the checks are green and then merge. When one is still open, the next line names it and does not say to merge.
- The command prints one JSON document and does not merge.
- Re-running `change ship` on an already archived change commits only that archive package, not other tracked edits.
- `change ship --wait` without `--push` does not poll GitHub for an archive commit that exists only locally.
