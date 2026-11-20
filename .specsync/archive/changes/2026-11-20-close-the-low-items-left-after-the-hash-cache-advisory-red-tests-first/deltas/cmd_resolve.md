## MODIFIED
### REQUIREMENT REQ-cmd-resolve-003

`resolve` SHALL read the local spec files it reads (8 MiB) only through the bounded project read of `util` (REQ-util-003), and the remote spec cache (8 MiB) only through the bounded read of its local state folder (REQ-util-006). A refused read, because the file leaves the project through a symbolic link, is a dangling link, is not a regular file, or is larger than the bound, SHALL be handled as that read's other read errors are, with a message that names the path and the reason, and SHALL never wait on the file or read past the bound.
A refused read of a local spec SHALL be reported with the path and the reason, never skipped silently with its `depends_on`.

Acceptance Criteria
- `tests/integration/project_reads.rs` `no_project_file_is_read_outside_the_bounded_read` finds no direct file read in this module's non-test source outside the allowed list
- `resolve` with an oversized spec names it and the reason.
