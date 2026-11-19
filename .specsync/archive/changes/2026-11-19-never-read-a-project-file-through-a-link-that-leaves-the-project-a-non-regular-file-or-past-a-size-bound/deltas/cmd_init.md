## ADDED

### REQUIREMENT REQ-cmd-init-009

`init` SHALL read `.gitignore`, `.gitattributes` and `.specsync/sdd.json` only through the bounded project read of `util` (REQ-util-003), with a bound of 4 MiB. A read refused there, because the file leaves the project through a symbolic link, is a dangling link, is not a regular file, or is larger than the bound, SHALL be handled as that read's other read errors are, with a message that names the path and the reason, and SHALL never wait on the file or read past the bound.

Acceptance Criteria
- With `.gitignore` a link to a file outside the project, `init` does not read or change that file.
- `tests/integration/project_reads.rs` `no_project_file_is_read_outside_the_bounded_read` finds no direct file read in this module's non-test source outside the allowed list
