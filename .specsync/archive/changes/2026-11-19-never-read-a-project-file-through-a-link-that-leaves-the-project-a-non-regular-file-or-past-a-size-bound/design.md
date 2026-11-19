---
change: never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound
artifact: design
---

# Design

**`util`.** It gains the bounded read, on the same `cap_std` handle as change 2's write functions.

- `read_project_file(root, path, max_bytes) -> io::Result<Option<Vec<u8>>>` takes the bound it was missing. It already opened without blocking and checked the file type on the handle. Now it also refuses a file whose length is over the bound, and it reads at most `max_bytes + 1` bytes, so a file that grows while it is read is refused too.
- `read_project_bytes` and `read_project_string` have the same contract as `std::fs::read` and `read_to_string`, a missing file is a not-found error, so each call site keeps its error handling.
- The refusal reasons are:
  - `<path> is a symbolic link that leaves the project`
  - `<path> is a dangling symbolic link`
  - `<path> is not a regular file`
  - `<path> is larger than <n> bytes`
- The bounds are named constants: `MAX_STATE_FILE_BYTES` (4 MiB), `MAX_SPEC_FILE_BYTES` and `MAX_SOURCE_FILE_BYTES` (8 MiB), and `MAX_PROJECT_INPUT_BYTES` (64 MiB).

**Call sites.** Each `fs::read`, `fs::read_to_string` or `File::open` of a project path becomes the matching `util` read with its bound, and keeps its existing error handling.
- Functions that had only a path gain the project root.
- `hash_file(root, path)` reads up to 8 MiB and hashes it. A refused or missing file is unreadable, which the cache already treats as changed.
- `HashCache::read` keeps its strict no-link handle and adds the 64 MiB bound.
- The remote spec cache keeps its strict check and reads with the 8 MiB bound.

**The guard.** An integration test scans every non-test source file for direct `fs::read`, `fs::read_to_string`, `File::open` and `read_to_end`, outside an allow-list:
- `util` itself;
- the confined readers that are already bounded;
- Git output;
- the merge-driver files;
- the paths named on the command line;
- the repository's own `.git`.

A new direct read of a project file fails it.

**Not changed:** the reads already inside the confined, bounded discovery.
