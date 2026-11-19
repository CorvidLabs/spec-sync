## ADDED

### REQUIREMENT REQ-cmd-scaffold-007

`scaffold` and `add-spec` SHALL create the spec file only through `create_project_file` of `util` (REQ-util-002) and its companions through the generator (REQ-generator-010), and `scaffold` SHALL print the error and exit 1 when the registry refuses the append (REQ-registry-003). A target that is a symbolic link to a regular file inside the project SHALL be written through. A target, or a directory on the way to it, that leaves the project through a symbolic link, a dangling link, and a target that is not a regular file SHALL be refused with an error that names the path, leaving the link and what it points at unchanged and creating nothing outside the project.

Acceptance Criteria
- A dangling link at `specs/<name>/<name>.spec.md`: `scaffold <name>` and `add-spec <name>` exit 1 naming the path and nothing is created at the link's target.
- `specs/<name>` linked to a directory outside the project: both exit 1 and nothing is created there.
- `.specsync/registry.toml` linked to a file outside the project: `scaffold <name>` exits 1 naming the registry and the outside file is unchanged.
- `specs/<name>` linked to a directory inside the project: the spec is created in that directory.
