---
spec: util.spec.md
---

## User Stories

- As a validator author, I want a shared edit-distance helper so near-miss source/spec references can produce useful suggestions without duplicating string comparison logic.
- As a rule/config author, I want user-provided regex patterns compiled with bounded resources so invalid or pathological patterns cannot crash validation.

## Acceptance Criteria

- `levenshtein` returns character-based edit distance for UTF-8 strings, including empty-string inputs.
- `safe_regex` returns a compiled regex for valid patterns within configured size limits.
- `safe_regex` returns `None` for invalid syntax or patterns that exceed regex/DFA size limits.
- Utility helpers remain dependency-light and do not read project configuration or filesystem state.

## Constraints

- Regex compilation must use explicit size and DFA limits to reduce ReDoS risk from configuration-supplied patterns.
- Helpers must be deterministic and side-effect free so they are safe to call from validation, parsing, and suggestion code.

## Out of Scope

- High-level validation policy decisions belong in validator, parser, or command modules.
- Regex matching semantics beyond safe compilation are handled by callers.

### REQ-util-001

The `util` module SHALL preserve truthful user-visible behavior for the pre-6.0 product fixes landed in this change.

Acceptance Criteria
- Related tests remain green.
- No intentional regression of SpecSync 6.0 lifecycle verbs.

### REQ-util-002

`read_project_file`, `write_project_file`, `create_project_file` and `remove_project_file` SHALL reach a project file only through a directory handle on the project root, resolving every component beneath it, so nothing they read, write, create or remove is outside the root. They SHALL follow a symbolic link, on a directory or on the file itself, whose target stays inside the root. They SHALL refuse, with an error that names the path and changing nothing, a path that leaves the root through a symbolic link (a link with an absolute target included) or through `..`, a dangling link, and a target that is not a regular file. `create_project_file` SHALL create only a file that does not exist yet and SHALL never create one through a link. `project_path_refusal` SHALL give the same reason without opening the file for writing.

Acceptance Criteria
- A file linked to a file outside the root: `write_project_file` and `read_project_file` return an error naming the path, and the outside file keeps its bytes.
- A dangling link: `write_project_file` and `create_project_file` return an error and nothing is created at its target.
- A link to a second link that leaves the root is refused like a direct one.
- A file linked to a regular file inside the root: `write_project_file` replaces that file's content, the link stays a link, and `read_project_file` returns the content.
- A directory component linked outside the root: nothing is created or written outside. Linked to a directory inside the root, the write lands in that directory.
- A directory at the file's path is refused.
- `create_project_file` returns `false` and changes nothing when a regular file is already there, and `remove_project_file` removes only a name inside the root.
- `project_path_refusal` returns the reason for each refused case and nothing for a regular, absent or inside-linked path.

### REQ-util-003

`read_project_file`, `read_project_bytes` and `read_project_string` SHALL read a file beneath the project root through the same directory handle as the other project file functions (REQ-util-002), with a byte bound the caller passes. They SHALL open the file without blocking, check on the open handle that it is a regular file, and read at most one byte past the bound. They SHALL follow a symbolic link whose target stays inside the root, and SHALL refuse, with an error that names the path and the reason, a link that leaves the root, a dangling link, a target that is not a regular file, and a file larger than the bound. `read_project_file` SHALL return nothing for a missing file; `read_project_bytes` and `read_project_string` SHALL report a missing file as not found, as `std::fs::read` and `std::fs::read_to_string` do. `util` SHALL name the bounds the callers use: 4 MiB for policy, config, state and change records, 8 MiB for specs, companions, templates, hook files, source and schema files, and 64 MiB for the transaction journal, the hash cache and other project inputs.

Acceptance Criteria
- A file linked outside the root, a dangling link, a FIFO and a directory are refused with the path and the reason; the FIFO read returns at once.
- A regular file one byte larger than the bound is refused naming the bound; a file exactly at the bound is read.
- A link to a regular file inside the root is read.
- A missing file is `None` from `read_project_file` and a not-found error from the other two.

### REQ-util-004

Inside a Git work tree, the project file functions SHALL confine a path to the work tree that holds the project root (`git rev-parse --show-toplevel`, asked once per root with `GIT_DIR` and `GIT_WORK_TREE` unset), so a monorepo subproject follows a link that stays inside its repository; outside a work tree, or when Git is not installed, the confinement SHALL stay the project root. Their messages SHALL name a path relative to the project root. An absolute symbolic link SHALL never be followed, and its refusal SHALL say it is an absolute link. A write to an existing file SHALL open it without blocking and without truncating, check on the open handle that it is a regular file, and only then empty and write it. `read_file_within` SHALL read a file within exactly a directory the person names, following a link that stays inside it and refusing one that leaves it, with the same bound. `create_new_project_dir` SHALL create a directory that must not exist yet, `is_project_file` SHALL say whether a path is a regular file the functions would read, `remove_empty_project_dir` SHALL remove an empty directory without following a link to it, and `find_project_files` SHALL list the files under a project directory through the same rule, starting at the project root itself when the directory is the root, without walking a link to a directory or the Git directory, and SHALL stop with the reason past an entry or a depth budget.

Acceptance Criteria
- A subproject at `packages/app` installs hooks through `CLAUDE.md -> ../../AGENTS.md` and reads its specs through `specs -> ../../specs/app`; a link that leaves the repository is still refused.
- A link to an absolute path inside the project is refused as an absolute link.
- A FIFO is refused without blocking; an outside directory's link out is refused by `read_file_within`; an empty directory reached through a link out is not removed.

### REQ-util-005

The project file functions SHALL refuse a path that resolves into the Git directory, directly or through links: a path with a `.git` component in any letter case or with trailing dots or spaces, a path under the work tree's Git directory (`git rev-parse --absolute-git-dir`, asked with the work tree's top folder), or a symbolic link whose canonical target lies under either, with the error `<path> leads into the Git directory`. Git runs hooks and reads its config from there, so a link that stays inside the project SHALL still never reach it. SpecSync's own writes into the Git directory, such as `hooks install` writing a Git hook, SHALL keep going through their own paths and are not project file writes.

Acceptance Criteria
- `hooks install` refuses `CLAUDE.md -> .git/hooks/post-checkout` and `AGENTS.md -> .GIT/config`, and both files keep their bytes.
- `change answer` refuses a committed `change.md -> ../../../.git/hooks/pre-commit`, and the hook keeps its bytes.
- With a Git directory named `gitstore` inside the work tree, a path under it, a link to a hook in it, and a path through a link to it are refused, the hook keeps its bytes, and a walk lists nothing from it.

### REQ-util-006

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

### REQ-util-007

The project file functions SHALL confine to the enclosing Git work tree's top folder only when the project root is that top folder, or when Git tracks a file under the project root (`git ls-files --error-unmatch -- ':(top,literal)<prefix>'`). Otherwise, including when Git cannot answer, they SHALL confine to the project root itself. Paths into the Git directory SHALL be refused either way (REQ-util-005).

Acceptance Criteria
- An untracked project unpacked inside another work tree refuses a link to a file of that work tree and leaves it unchanged.
- A tracked monorepo subproject still follows a link that stays inside its repository.

### REQ-util-008

The read and write scans (REQ-util-002, REQ-util-003) SHALL scan exactly the source files Cargo compiled into the `specsync` binary, as its dep-info lists them, and SHALL fail when that list cannot be read. They SHALL leave out only `#[cfg(test)]` items, found by a lexer that ignores braces in comments, strings, raw and byte strings and character literals. They SHALL also report cap-std opens (`open_ambient_dir`, `Dir::open`, `File::open_ambient` and `open_with`), directory listings (`read_dir` and `WalkDir::new`), renames and copies, `tempfile_in`, `NamedTempFile::new_in`, `persist_noclobber`, hard and symbolic links, `set_len`, a free function imported from `std::fs` by name, and `std::fs` imported under another name. A third scan SHALL report a spawned Git argument that writes files (`--output`, `worktree`, `checkout`, `restore`, `stash`, `clean`, `reset`, `bundle`, `format-patch`, `-o`, `apply`, `merge`, `switch`, `config`) unless it says why no project file is written, and a fourth SHALL report a spawned command other than Git unless it says the same.

Acceptance Criteria
- A production read in an `include!`d file named `tests.rs`, and a read after a string literal holding `{`, are reported until they go through the bounded read or are marked.
- An unmarked `read_dir`, a Git `merge` or `config` call and a spawned command other than Git are reported until they go through the confined helpers or are marked.

### REQ-util-009

The change read cache (passing verdicts for archived change history) SHALL be kept only in `specsync/repositories/<key>` in the user's cache folder, with the key taken from a digest of the canonical Git common directory, so worktrees of one repository share it. It SHALL never be kept in a Git directory, which an archive of a working copy carries, and a user cache folder inside the project SHALL be refused as for the other state (REQ-util-006).
It SHALL be read and written only through the handles of `LocalState` (REQ-util-006): opened without following a link and without blocking, bounded, and replaced by rename, so a link or a non-regular file at its path is refused, never read through or waited on.

Acceptance Criteria
- The read cache path of a Git work tree is outside the clone, and two worktrees of one repository get the same path.
- A FIFO at the read cache path is refused without waiting.

### REQ-util-010

`read_project_dir` SHALL list one directory beneath the project root through the project rule (REQ-util-002), so a directory that leaves the project through a link at it or on the way to it, a dangling link, or the Git directory is refused with the path, and `..` is refused before the filesystem is asked.
SpecSync's own record folders (`.specsync/archive/changes`, a change's `deltas`, a spec's `history/`) SHALL be listed only through `read_project_dir`, so a committed link there is refused with the folder named, and no name in the folder it leads to is listed or printed.

Acceptance Criteria
- A schema folder, a specs folder or a missing file's folder that leads out of the project is never listed.
- With `.specsync/archive/changes` committed as a link out of the project, `change audit --archives` refuses it, naming the folder, and prints no name from outside.

### REQ-util-011

`read_typed_file` SHALL read a file the person names on the command line: it SHALL follow links, since the person chose the path, but SHALL open the file without blocking, check on the open handle that it is a regular file, and read at most one byte past the bound the caller passes. A FIFO, a device, a directory or a larger file SHALL be refused with the path and the reason.

Acceptance Criteria
- A FIFO is refused without waiting, and a file one byte past the bound is refused.
- A link to a regular file is read.

