## MODIFIED

### REQUIREMENT REQ-util-006

SpecSync SHALL keep the state it writes for itself about a project (the hash cache, the transaction journal and the remote spec cache) only in the user's cache folder, never in the work tree or in a Git directory, where a repository or an archive of one can put a file.
- `project_state_dir` SHALL be `specsync/projects/<key>` in the user's cache folder: `$XDG_CACHE_HOME` when it is an absolute path, else `~/Library/Caches` on macOS, `%LOCALAPPDATA%` on Windows and `~/.cache` elsewhere. The key SHALL be taken from a digest of the canonical project root.
- A user cache folder that lies inside the canonical project root, or inside the folder the project file functions confine to, SHALL be refused with the reason, as when no folder can be found.
- `LocalState` SHALL open that folder without following a link at any name it creates. It SHALL read a file there without blocking, check on the open handle that it is a regular file, and read at most one byte past the bound the caller passes. It SHALL write a file by creating a new temporary file there and renaming it over the name, syncing both when asked to be durable, and SHALL remove a file by unlinking its name.
- When no such folder can be found, or it is refused, `project_state_dir` SHALL say why, and nothing SHALL be kept in the work tree or a Git directory instead.

Acceptance Criteria
- In a Git work tree the folder is under the user's cache folder, not the Git directory; two projects get different folders; a forged cache, journal or remote spec entry an archive ships under its `.git` is never read.
- A user cache folder inside the project is refused.
- A link SpecSync did not create is never followed into or out of the folder; a FIFO there is refused without blocking.

### REQUIREMENT REQ-util-008

The read and write scans (REQ-util-002, REQ-util-003) SHALL scan exactly the source files Cargo compiled into the `specsync` binary, as its dep-info lists them, and SHALL fail when that list cannot be read. They SHALL leave out only `#[cfg(test)]` items, found by a lexer that ignores braces in comments, strings, raw and byte strings and character literals. They SHALL also report cap-std opens (`open_ambient_dir`, `Dir::open`, `File::open_ambient` and `open_with`), renames and copies, `tempfile_in`, `NamedTempFile::new_in`, `persist_noclobber`, hard and symbolic links, `set_len`, a free function imported from `std::fs` by name, and `std::fs` imported under another name. A third scan SHALL report a spawned Git argument that writes files (`--output`, `worktree`, `checkout`, `restore`, `stash`, `clean`, `reset`, `bundle`, `format-patch`, `-o`) unless it says why no project file is written.

Acceptance Criteria
- A production read in an `include!`d file named `tests.rs`, and a read after a string literal holding `{`, are reported until they go through the bounded read or are marked.
