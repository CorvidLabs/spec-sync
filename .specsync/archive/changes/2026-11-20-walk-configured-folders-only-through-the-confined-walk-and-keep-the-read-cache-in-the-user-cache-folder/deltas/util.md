## ADDED

### REQUIREMENT REQ-util-009

The change read cache (passing verdicts for archived change history) SHALL be kept only in `specsync/repositories/<key>` in the user's cache folder, with the key taken from a digest of the canonical Git common directory, so worktrees of one repository share it. It SHALL never be kept in a Git directory, which an archive of a working copy carries, and a user cache folder inside the project SHALL be refused as for the other state (REQ-util-006).

Acceptance Criteria
- The read cache path of a Git work tree is outside the clone, and two worktrees of one repository get the same path.

### REQUIREMENT REQ-util-010

`read_project_dir` SHALL list one directory beneath the project root through the project rule (REQ-util-002), so a directory that leaves the project through a link at it or on the way to it, a dangling link, or the Git directory is refused with the path, and `..` is refused before the filesystem is asked.

Acceptance Criteria
- A schema folder, a specs folder or a missing file's folder that leads out of the project is never listed.
