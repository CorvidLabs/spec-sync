## ADDED

### REQUIREMENT REQ-compact-003

`compact` SHALL write project files only through the confined project write functions of `util` (REQ-util-002), or through a write whose path was refused as a link just before; any other direct write SHALL say, on the write or the line above it in the same statement, why its path is not a project file (a temporary or Git directory, a test barrier, a file Git hands the command). It SHALL find spec files only through the confined, budgeted walk of `validator` (REQ-validator-050), so a `specs` link that leaves the repository, or a tree past the walk's budget, stops it with the reason instead of being walked.

Acceptance Criteria
- `tests/integration/project_writes.rs` `no_project_file_is_written_outside_the_confined_helpers` finds no unmarked direct write in this module's non-test source
