## ADDED

### REQUIREMENT REQ-cmd-import-003

`import`, for one item and for a batch, SHALL create each spec file only through `create_project_file` of `util` (REQ-util-002) and its companions through the generator (REQ-generator-010). A target that is a symbolic link to a regular file inside the project SHALL be written through. A target, or a directory on the way to it, that leaves the project through a symbolic link, a dangling link, and a target that is not a regular file SHALL be refused with an error that names the path, leaving the link and what it points at unchanged and creating nothing outside the project.

Acceptance Criteria
- A dangling link at the spec path of an imported item: the item fails with an error naming the path and nothing is created at the link's target.
- The spec directory of an item linked to a directory outside the project: the item fails and nothing is created there.
- An item with no link is imported as before.
