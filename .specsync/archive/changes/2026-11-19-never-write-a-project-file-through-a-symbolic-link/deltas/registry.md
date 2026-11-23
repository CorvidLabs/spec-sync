## ADDED

### REQUIREMENT REQ-registry-003

`register_module` SHALL read and rewrite the local registry only through the project file functions of `util` (REQ-util-002), and SHALL return a refused registry path as an error instead of reporting that nothing was added. A target that is a symbolic link to a regular file inside the project SHALL be written through. A target, or a directory on the way to it, that leaves the project through a symbolic link, a dangling link, and a target that is not a regular file SHALL be refused with an error that names the path, leaving the link and what it points at unchanged and creating nothing outside the project.

Acceptance Criteria
- `.specsync/registry.toml` linked to a file outside the project: `register_module` returns an error naming it and the outside file is unchanged.
- A regular registry gains the module line as before, and a module already listed is not added twice.
