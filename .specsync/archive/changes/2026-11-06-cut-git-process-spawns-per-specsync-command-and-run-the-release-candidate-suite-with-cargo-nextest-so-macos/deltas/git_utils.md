## ADDED

### REQUIREMENT REQ-git-utils-008

Within one SpecSync process, repository facts that the process does not itself change SHALL be asked of git at most once per canonical repository root: the HEAD commit, the working-directory prefix, whether the root is inside a work tree, whether the repository is shallow, the origin default branch and its commit, the index path, the shared index path, `core.fsmonitor`, and the core line-ending, symlink and file-mode settings. A failed git query SHALL NOT be memoized. After the process itself commits, writes the index, moves a ref or switches branches in that repository, the memoized facts for that root SHALL be discarded so the next read asks git again.

#### Acceptance Criteria

- Two reads of the same fact in one process spawn one git process.
- A read after the process commits, writes the index, runs `update-ref` or switches branches returns the new value.
- A git failure is retried on the next read and still returns the existing safe default.
- Command output is byte-identical to the unmemoized behavior.

### REQUIREMENT REQ-git-utils-009

Within one SpecSync process, committed blob contents and tree listings read at a fixed commit SHALL be served by at most one `git cat-file --batch` child per canonical repository root, started on first use and closed when the process ends. Reads named through `HEAD` SHALL resolve HEAD through the repository-fact memo first. A missing object or path SHALL produce the same absent result as a per-file `git show` or `git ls-tree` read.

#### Acceptance Criteria

- Reading many blobs at fixed commits in one process starts one `git cat-file --batch` child.
- Batched reads equal per-file reads byte for byte, including empty files and binary content.
- A missing path or object reads as absent, as before.
- A read through `HEAD` after the process commits returns the new commit's content.
