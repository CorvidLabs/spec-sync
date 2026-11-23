## ADDED

### REQUIREMENT REQ-cmd-migrate-005

`migrate` SHALL refuse before its first write, changing nothing, when a file it would write, move, copy or remove, a directory it would create, or a spec file it would back up or rewrite leaves the project through a symbolic link, is a dangling link, or is not a regular file, and SHALL name each such path. Every write, copy and removal it then makes SHALL go through the project file functions of `util` (REQ-util-002).

Acceptance Criteria
- A dangling link at `.specsync/.gitignore` in a 3.x project: `migrate` exits non-zero naming it, creates nothing at the link's target, and leaves `specsync.json`, the registry and every spec as they were.
- A spec file linked to a file outside the project: `migrate` exits non-zero naming it and the outside file is unchanged.
- `.specsync` linked to a directory outside the project: `migrate` exits non-zero and nothing is created there.
- A 3.x project with no link migrates as before.
