## ADDED

### REQUIREMENT REQ-generator-010

`generate_companion_files_for_spec` and `generate_companion_files_from_template` SHALL create each companion file only through `create_project_file` of `util` (REQ-util-002). A companion that already exists SHALL be left as it is. A companion whose path is a dangling link, or leaves the project through a link, SHALL be refused with an error on stderr that names the path, without stopping the other companions, and nothing SHALL be created outside the project.

Acceptance Criteria
- A dangling link at `specs/<module>/tasks.md` to a path outside the project: nothing is created at its target, the other companions are created, and stderr names `specs/<module>/tasks.md`.
- The spec directory linked to a directory outside the project: no companion is created there.
- An existing companion keeps its content.
