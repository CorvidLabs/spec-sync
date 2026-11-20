## ADDED

### REQUIREMENT REQ-validator-050

`find_spec_files` SHALL list spec files through `util`'s `find_project_files` (REQ-util-004), with the entry and depth budget coverage uses. A `specs` directory that leads out of the repository, and a tree past the budget, SHALL stop the walk with `spec discovery stopped:` and the reason; a command SHALL then exit 1, and `find_spec_files_checked` SHALL return the reason to its caller instead.

Acceptance Criteria
- A `specs` link to an outside directory stops `deps` with the reason, and nothing found outside is named.
- A tree deeper than 256 directories stops `deps` with the reason.
- A `specs_dir` of `.` lists the spec files under the project root, outside Git and at the top of a work tree.

### REQUIREMENT REQ-validator-051

`validate_spec` SHALL read the spec, its `requirements.md` and companion files within the spec bound, through the bounded project read for a file in the project and within the spec's own directory for a temporary effective contract; it SHALL read each listed source within the source bound through the bounded project read.

Acceptance Criteria
- `tests/integration/project_reads.rs` `no_project_file_is_read_outside_the_bounded_read` finds no unmarked direct read in this module's non-test source.
