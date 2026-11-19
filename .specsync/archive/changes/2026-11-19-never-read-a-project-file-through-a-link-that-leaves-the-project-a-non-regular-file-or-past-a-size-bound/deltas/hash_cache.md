## ADDED

### REQUIREMENT REQ-hash-cache-008

The hash cache SHALL read `.specsync/hashes.json` (64 MiB, still refusing any link as REQ-hash-cache-007 requires) and every file it hashes or reads to classify a spec (8 MiB) only through the bounded project read of `util` (REQ-util-003), with a bound of the one named for each file. A read refused there, because the file leaves the project through a symbolic link, is a dangling link, is not a regular file, or is larger than the bound, SHALL be handled as that read's other read errors are, with a message that names the path and the reason, and SHALL never wait on the file or read past the bound. A file it cannot hash is treated as changed, as before.

Acceptance Criteria
- With a spec's `requirements.md` a FIFO, `check` finishes without waiting on it.
- With `.specsync/hashes.json` one byte over 64 MiB, it loads as an empty cache without reading it whole.
- `tests/integration/project_reads.rs` `no_project_file_is_read_outside_the_bounded_read` finds no direct file read in this module's non-test source outside the allowed list
