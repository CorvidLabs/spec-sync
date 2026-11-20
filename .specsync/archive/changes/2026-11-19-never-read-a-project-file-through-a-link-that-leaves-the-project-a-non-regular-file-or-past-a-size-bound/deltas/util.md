## ADDED

### REQUIREMENT REQ-util-003

`read_project_file`, `read_project_bytes` and `read_project_string` SHALL read a file beneath the project root through the same directory handle as the other project file functions (REQ-util-002), with a byte bound the caller passes. They SHALL open the file without blocking, check on the open handle that it is a regular file, and read at most one byte past the bound. They SHALL follow a symbolic link whose target stays inside the root, and SHALL refuse, with an error that names the path and the reason, a link that leaves the root, a dangling link, a target that is not a regular file, and a file larger than the bound. `read_project_file` SHALL return nothing for a missing file; `read_project_bytes` and `read_project_string` SHALL report a missing file as not found, as `std::fs::read` and `std::fs::read_to_string` do. `util` SHALL name the bounds the callers use: 4 MiB for policy, config, state and change records, 8 MiB for specs, companions, templates, hook files, source and schema files, and 64 MiB for the transaction journal, the hash cache and other project inputs.

Acceptance Criteria
- A file linked outside the root, a dangling link, a FIFO and a directory are refused with the path and the reason; the FIFO read returns at once.
- A regular file one byte larger than the bound is refused naming the bound; a file exactly at the bound is read.
- A link to a regular file inside the root is read.
- A missing file is `None` from `read_project_file` and a not-found error from the other two.
