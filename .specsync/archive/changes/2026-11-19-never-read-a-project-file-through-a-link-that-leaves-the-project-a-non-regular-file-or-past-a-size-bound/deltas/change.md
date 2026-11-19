## ADDED

### REQUIREMENT REQ-change-161

The change lifecycle SHALL read `.specsync/sdd.json` and every change record under `.specsync/changes/` and `.specsync/archive/` (4 MiB), `.specsync/change-transaction.json` and a delivery file whose digest it compares (64 MiB), and the spec, requirements, companion and history files it reads (8 MiB) only through the bounded project read of `util` (REQ-util-003), with a bound of the one named for each file. A read refused there, because the file leaves the project through a symbolic link, is a dangling link, is not a regular file, or is larger than the bound, SHALL be handled as that read's other read errors are, with a message that names the path and the reason, and SHALL never wait on the file or read past the bound.

Acceptance Criteria
- With `.specsync/sdd.json` a FIFO, `change status --json` finishes without waiting on it and reports the policy as unreadable, naming the path.
- With `.specsync/sdd.json` a link to a file outside the project, that file's bytes are not read: the error names the link as leaving the project.
- With `.specsync/sdd.json` one byte over 4 MiB, it is refused naming the bound, without reading it whole.
- With `.specsync/change-transaction.json` a FIFO or a link to a file outside the project, a lifecycle command finishes without waiting on it and reports the journal invalid, naming the path.
- `tests/integration/project_reads.rs` `no_project_file_is_read_outside_the_bounded_read` finds no direct file read in this module's non-test source outside the allowed list
