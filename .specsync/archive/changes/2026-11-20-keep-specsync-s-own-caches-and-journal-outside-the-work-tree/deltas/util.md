## ADDED

### REQUIREMENT REQ-util-006

SpecSync SHALL keep the state it writes for itself about a project (the hash cache, the transaction journal and the remote spec cache) only outside the project's work tree, where a repository cannot put a file.
- In a Git work tree, `project_state_dir` SHALL be `<git rev-parse --absolute-git-dir>/specsync/<key>`, with the key taken from a digest of the project root's path inside the work tree.
- With no work tree, it SHALL be `specsync/projects/<key>` in the user's cache folder: `$XDG_CACHE_HOME` when it is an absolute path, else `~/Library/Caches` on macOS, `%LOCALAPPDATA%` on Windows and `~/.cache` elsewhere. The key SHALL be taken from a digest of the canonical project root.
- `LocalState` SHALL open that folder without following a link at any name it creates. It SHALL read a file there without blocking, check on the open handle that it is a regular file, and read at most one byte past the bound the caller passes. It SHALL write a file by creating a new temporary file there and renaming it over the name, syncing both when asked to be durable, and SHALL remove a file by unlinking its name.
- When no such folder can be found, `project_state_dir` SHALL say why, and nothing SHALL be kept in the work tree instead.

Acceptance Criteria
- In a Git work tree the folder is under the Git directory; two projects in one work tree get different folders; with no work tree it is under the user's cache folder.
- A link SpecSync did not create is never followed into or out of the folder; a FIFO there is refused without blocking.

### REQUIREMENT REQ-util-007

The project file functions SHALL confine to the enclosing Git work tree's top folder only when the project root is that top folder, or when Git tracks a file under the project root (`git ls-files --error-unmatch -- ':(top,literal)<prefix>'`). Otherwise, including when Git cannot answer, they SHALL confine to the project root itself. Paths into the Git directory SHALL be refused either way (REQ-util-005).

Acceptance Criteria
- An untracked project unpacked inside another work tree refuses a link to a file of that work tree and leaves it unchanged.
- A tracked monorepo subproject still follows a link that stays inside its repository.

### REQUIREMENT REQ-util-008

The read and write scans (REQ-util-002, REQ-util-003) SHALL scan exactly the source files Cargo compiled into the `specsync` binary, as its dep-info lists them, and SHALL fail when that list cannot be read. They SHALL leave out only `#[cfg(test)]` items, found by a lexer that ignores braces in comments, strings, raw and byte strings and character literals. They SHALL also report cap-std opens, renames and copies, `tempfile_in`, `NamedTempFile::new_in`, `persist_noclobber`, hard and symbolic links, `set_len`, and a free function imported from `std::fs` by name. A third scan SHALL report a spawned Git argument that writes files (`--output`, `worktree`, `checkout`, `restore`, `stash`, `clean`, `reset`, `archive`, `bundle`) unless it says why no project file is written.

Acceptance Criteria
- A production read in an `include!`d file named `tests.rs`, and a read after a string literal holding `{`, are reported until they go through the bounded read or are marked.
