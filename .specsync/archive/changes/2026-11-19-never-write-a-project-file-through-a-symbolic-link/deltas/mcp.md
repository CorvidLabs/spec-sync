## ADDED

### REQUIREMENT REQ-mcp-011

`mcp lock --write` SHALL write `.specsync/mcp-tools.lock.json` only through `write_project_file` of `util` (REQ-util-002). A target that is a symbolic link to a regular file inside the project SHALL be written through. A target, or a directory on the way to it, that leaves the project through a symbolic link, a dangling link, and a target that is not a regular file SHALL be refused with an error that names the path, leaving the link and what it points at unchanged and creating nothing outside the project.

Acceptance Criteria
- The lock file linked to a file outside the project: `mcp lock --write` fails naming `.specsync/mcp-tools.lock.json` and the outside file is unchanged.
- A dangling lock-file link creates nothing at its target.
- The lock file linked to a regular file inside the project is written through.
- With no link the lock is written as before and `mcp diff` accepts it.
