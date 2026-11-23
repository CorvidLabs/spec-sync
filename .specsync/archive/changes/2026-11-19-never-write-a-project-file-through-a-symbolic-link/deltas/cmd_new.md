## ADDED

### REQUIREMENT REQ-cmd-new-006

`new` SHALL create the spec file only through `create_project_file` of `util` (REQ-util-002) and its companions through the generator (REQ-generator-010). A target that is a symbolic link to a regular file inside the project SHALL be written through. A target, or a directory on the way to it, that leaves the project through a symbolic link, a dangling link, and a target that is not a regular file SHALL be refused with an error that names the path, leaving the link and what it points at unchanged and creating nothing outside the project.

Acceptance Criteria
- A dangling link at `specs/<name>/<name>.spec.md`: `new <name>` exits 1 naming the path and nothing is created at the link's target.
- `specs/<name>` linked to a directory outside the project: `new <name>` exits 1 and nothing is created there.
