## MODIFIED

### REQUIREMENT REQ-cmd-resolve-002

The remote spec cache SHALL be read and written only in `remote-specs/` in the project's local state folder outside the work tree (REQ-util-006), under a name taken from a digest of the repository and spec path, so neither its place nor its names come from the repository. A `.specsync-cache/remote-specs/` in the work tree SHALL never be read. When no local state folder can be found, `resolve --remote` SHALL say so once on stderr and use the fetched content as it does with no cache.

Acceptance Criteria
- An entry committed under `.specsync-cache/remote-specs/` with forged content is never returned by the cache.
- A stored entry is found again under the local state folder.

### REQUIREMENT REQ-cmd-resolve-003

`resolve` SHALL read the local spec files it reads (8 MiB) only through the bounded project read of `util` (REQ-util-003), and the remote spec cache (8 MiB) only through the bounded read of its local state folder (REQ-util-006). A refused read, because the file leaves the project through a symbolic link, is a dangling link, is not a regular file, or is larger than the bound, SHALL be handled as that read's other read errors are, with a message that names the path and the reason, and SHALL never wait on the file or read past the bound.

Acceptance Criteria
- `tests/integration/project_reads.rs` `no_project_file_is_read_outside_the_bounded_read` finds no direct file read in this module's non-test source outside the allowed list
