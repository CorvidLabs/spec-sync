## ADDED

### REQUIREMENT REQ-cmd-change-038

The `change` command handlers SHALL read `.specsync/sdd.json` and the change records they read (`finalization.json`, `verification.json`, `review.json`) only through the bounded project read of `util` (REQ-util-003), with a bound of 4 MiB. A read refused there, because the file leaves the project through a symbolic link, is a dangling link, is not a regular file, or is larger than the bound, SHALL be handled as that read's other read errors are, with a message that names the path and the reason, and SHALL never wait on the file or read past the bound.

Acceptance Criteria
- With `.specsync/sdd.json` a FIFO, `change audit` finishes without waiting on it.
- `tests/integration/project_reads.rs` `no_project_file_is_read_outside_the_bounded_read` finds no direct file read in this module's non-test source outside the allowed list
