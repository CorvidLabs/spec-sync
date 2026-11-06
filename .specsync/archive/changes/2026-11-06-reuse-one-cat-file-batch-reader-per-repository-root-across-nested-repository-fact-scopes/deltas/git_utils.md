## MODIFIED

### REQUIREMENT REQ-git-utils-010

Within one SpecSync process, committed blob contents and tree listings read at a fixed commit SHALL be served by at most one `git cat-file --batch` child per canonical repository root, started on first use and closed when the process ends. Reads named through `HEAD` SHALL resolve HEAD through the repository-fact memo first. A missing object or path SHALL produce the same absent result as a per-file `git show` or `git ls-tree` read.

#### Acceptance Criteria

- Reading many blobs at fixed commits in one process starts one `git cat-file --batch` child.
- A read in a scope nested inside an open scope for the same root uses the reader the outer scope already started, and a reader a nested scope starts is the one later reads in the outer scope use.
- Batched reads equal per-file reads byte for byte, including empty files and binary content.
- A missing path or object reads as absent, as before.
- A read through `HEAD` after the process commits returns the new commit's content.
