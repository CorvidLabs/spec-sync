## ADDED

### REQUIREMENT REQ-exports-011

Export extraction SHALL read source files and the TypeScript files it follows for re-exports only through the bounded project read of `util` (REQ-util-003), with a bound of 8 MiB. A read refused there, because the file leaves the project through a symbolic link, is a dangling link, is not a regular file, or is larger than the bound, SHALL be handled as that read's other read errors are, with a message that names the path and the reason, and SHALL never wait on the file or read past the bound.

Acceptance Criteria
- With a source file a spec lists linked to a file outside the project, `check` does not read that file's exports.
- `tests/integration/project_reads.rs` `no_project_file_is_read_outside_the_bounded_read` finds no direct file read in this module's non-test source outside the allowed list
