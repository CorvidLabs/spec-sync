## MODIFIED
### REQUIREMENT REQ-util-006

SpecSync SHALL keep the state it writes for itself about a project (the hash cache and the remote spec cache) only in the user's cache folder, and its transaction journal only in the user's state folder, never in the work tree or in a Git directory, where a repository or an archive of one can put a file.
- `project_state_dir` SHALL be `specsync/projects/<key>` in the user's cache folder: `$XDG_CACHE_HOME` when it is an absolute path, else `~/Library/Caches` on macOS, `%LOCALAPPDATA%` on Windows and `~/.cache` elsewhere. The key SHALL be taken from a digest of the canonical project root.
- The journal's folder SHALL be `specsync/projects/<key>` in the user's state folder: `$XDG_STATE_HOME` when it is an absolute path, else `~/Library/Application Support` on macOS, `%LOCALAPPDATA%` on Windows and `~/.local/state` elsewhere, with the same key and the same refusals.
- A user cache or state folder that lies inside the canonical project root, or inside the folder the project file functions confine to, or a project folder that would itself lie inside the project, SHALL be refused with the reason, as when no folder can be found.
- On Unix, the `specsync` folder and each folder beneath it on the way to the project's folder SHALL be refused, with the path and the reason, when the current user does not own it or its group or others can write it, as `ssh` refuses such a folder.
- `LocalState` SHALL open that folder without following a link at any name it creates. It SHALL read a file there without blocking, check on the open handle that it is a regular file, and read at most one byte past the bound the caller passes. It SHALL write a file by creating a new temporary file there and renaming it over the name, syncing both when asked to be durable, and SHALL remove a file by unlinking its name.
- When no such folder can be found, or it is refused, `project_state_dir` SHALL say why, and nothing SHALL be kept in the work tree or a Git directory instead.

Acceptance Criteria
- In a Git work tree the folder is under the user's cache folder, not the Git directory; two projects get different folders; a forged cache, journal or remote spec entry an archive ships under its `.git` is never read.
- A user cache folder inside the project is refused.
- A link SpecSync did not create is never followed into or out of the folder; a FIFO there is refused without blocking.
- A project at `<cache>/specsync` is refused.
- A project folder with mode 0777 is refused, and `check` validates every spec.
- The journal's path is not under the user's cache folder.

### REQUIREMENT REQ-util-008

The read and write scans (REQ-util-002, REQ-util-003) SHALL scan exactly the source files Cargo compiled into the `specsync` binary, as its dep-info lists them, and SHALL fail when that list cannot be read. They SHALL leave out only `#[cfg(test)]` items, found by a lexer that ignores braces in comments, strings, raw and byte strings and character literals. They SHALL also report cap-std opens (`open_ambient_dir`, `Dir::open`, `File::open_ambient` and `open_with`), directory listings (`read_dir` and `WalkDir::new`), renames and copies, `tempfile_in`, `NamedTempFile::new_in`, `persist_noclobber`, hard and symbolic links, `set_len`, a free function imported from `std::fs` by name, and `std::fs` imported under another name. A third scan SHALL report a spawned Git argument that writes files (`--output`, `worktree`, `checkout`, `restore`, `stash`, `clean`, `reset`, `bundle`, `format-patch`, `-o`, `apply`, `merge`, `switch`, `config`) unless it says why no project file is written, and a fourth SHALL report a spawned command other than Git unless it says the same.

Acceptance Criteria
- A production read in an `include!`d file named `tests.rs`, and a read after a string literal holding `{`, are reported until they go through the bounded read or are marked.
- An unmarked `read_dir`, a Git `merge` or `config` call and a spawned command other than Git are reported until they go through the confined helpers or are marked.

### REQUIREMENT REQ-util-009

The change read cache (passing verdicts for archived change history) SHALL be kept only in `specsync/repositories/<key>` in the user's cache folder, with the key taken from a digest of the canonical Git common directory, so worktrees of one repository share it. It SHALL never be kept in a Git directory, which an archive of a working copy carries, and a user cache folder inside the project SHALL be refused as for the other state (REQ-util-006).
It SHALL be read and written only through the handles of `LocalState` (REQ-util-006): opened without following a link and without blocking, bounded, and replaced by rename, so a link or a non-regular file at its path is refused, never read through or waited on.

Acceptance Criteria
- The read cache path of a Git work tree is outside the clone, and two worktrees of one repository get the same path.
- A FIFO at the read cache path is refused without waiting.

### REQUIREMENT REQ-util-010

`read_project_dir` SHALL list one directory beneath the project root through the project rule (REQ-util-002), so a directory that leaves the project through a link at it or on the way to it, a dangling link, or the Git directory is refused with the path, and `..` is refused before the filesystem is asked.
SpecSync's own record folders (`.specsync/archive/changes`, a change's `deltas`, a spec's `history/`) SHALL be listed only through `read_project_dir`, so a committed link there is refused with the folder named, and no name in the folder it leads to is listed or printed.

Acceptance Criteria
- A schema folder, a specs folder or a missing file's folder that leads out of the project is never listed.
- With `.specsync/archive/changes` committed as a link out of the project, `change audit --archives` refuses it, naming the folder, and prints no name from outside.

## ADDED

### REQUIREMENT REQ-util-011

`read_typed_file` SHALL read a file the person names on the command line: it SHALL follow links, since the person chose the path, but SHALL open the file without blocking, check on the open handle that it is a regular file, and read at most one byte past the bound the caller passes. A FIFO, a device, a directory or a larger file SHALL be refused with the path and the reason.

Acceptance Criteria
- A FIFO is refused without waiting, and a file one byte past the bound is refused.
- A link to a regular file is read.
