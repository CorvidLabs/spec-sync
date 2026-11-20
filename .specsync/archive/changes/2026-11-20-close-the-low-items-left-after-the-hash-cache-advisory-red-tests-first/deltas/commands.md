## MODIFIED
### REQUIREMENT REQ-commands-014

The shared command helpers SHALL read the spec files they read for frontmatter, ownership and inline ignores only through the bounded project read of `util` (REQ-util-003), with a bound of 8 MiB. A read refused there, because the file leaves the project through a symbolic link, is a dangling link, is not a regular file, or is larger than the bound, SHALL be handled as that read's other read errors are, with a message that names the path and the reason, and SHALL never wait on the file or read past the bound.
Under `--only-status` or `--exclude-status`, a spec whose read is refused SHALL be kept, so validation reports the refusal with the path and the reason, never dropped as if its status did not match.

Acceptance Criteria
- `tests/integration/project_reads.rs` `no_project_file_is_read_outside_the_bounded_read` finds no direct file read in this module's non-test source outside the allowed list
- `check --only-status active` with an oversized spec lists that spec with its refusal and fails.
