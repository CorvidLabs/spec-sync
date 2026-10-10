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
- A directory under `.specsync/changes/` that holds a file or symlink but no `state.json` is still open, and the command is `specsync change status <id>`. That command says to restore state.json or remove the stray directory. A directory with no file or symlink is not an open change.
- `change ship` recommends `change ship` only for a workflow-v2 verifying change whose review verdict is pass and whose reviewed commit is HEAD. A block, a legacy record, or a review of another commit is told to run `change status`.
