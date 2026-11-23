---
change: keep-specsync-s-own-caches-and-journal-outside-the-work-tree
artifact: design
---

# Design

## Local state outside the work tree (F1, F2, F3)

New module `src/util/local_state.rs`, a child module of `util`.

**`project_state_dir(root)` says where the state lives:**
- **In a Git work tree:** `<absolute-git-dir>/specsync/<key>`. The key is the first 16 hex digits of SHA-256 over the project root's path in the work tree.
- **With no work tree:** `<user cache>/specsync/projects/<key>`. The key is the same digest over the canonical project root.
  - The user cache is `$XDG_CACHE_HOME` when it is set to an absolute path; otherwise `~/Library/Caches` on macOS, `%LOCALAPPDATA%` on Windows, and `~/.cache` elsewhere.
  - Unit tests use a folder in the system temporary directory, so they never touch the user's cache.

**`LocalState`** is a directory handle on that folder, opened without following a link at any name SpecSync creates (`specsync`, the key, `projects`).
- **Reads** are non-blocking, type-checked on the handle and bounded.
- **Writes** create a new temporary file and rename it over the name. A durable write also syncs the file and the folder.
- **A remove** unlinks the name.

**The three files there:**
- **The hash cache: `hashes.json`.** The work tree's `.specsync/hashes.json` is never looked at, so change 5's tracked check and its `git` processes go. `check` prints the real path in its "Cache:" line, and `rehash` prints where it wrote. With no state folder, `check` warns once and validates everything, and `rehash` exits 1.
- **The journal: `change-transaction.json`.** It is written, replayed and removed only there. A `.specsync/change-transaction.json` in the work tree, of any kind, makes the lifecycle lock refuse with its path and how to clear it, and it is never opened. Change 4's per-entry rule stays.
- **The remote spec cache: `remote-specs/<sha256 of repo and path>`.** The work tree's `.specsync-cache` is never looked at.

## Confinement (F7)

`work_tree_confinement` keeps the work tree's top folder only when the prefix is empty, or when `git ls-files --error-unmatch -- ':(top,literal)<prefix>'` exits 0. Otherwise the top is the project root. The Git directory is kept for the refusal either way. The local state key uses the real prefix.

## The hooks folder (F6)

`git_hooks_dir` returns a `HooksDir`: a handle reached component by component without following any link. It starts from one of these bases:
- the canonical Git directory, when the hooks path lies in it;
- the canonical work-tree top, when it lies there;
- otherwise the canonical parent of the user's own folder.

`pre-commit` is read, replaced, created, made executable and removed through that handle:
- **read:** no-follow, non-blocking, a type check on the handle, 8 MiB;
- **replace:** open without create, check the type, truncate, write;
- **create:** exclusive create;
- **make executable:** `fchmod` on the handle;
- **remove:** unlink.

A link on the way, or at `pre-commit`, is refused with the path named.

## The scans (F4)

- **Which files.** The read and write scans take the files to scan from Cargo's dep-info for the binary. They fail if it cannot be read or does not list `src/main.rs`.
- **`#[cfg(test)]` items** are found by a lexer that skips line and block comments, normal, raw and byte strings, and character literals.
- **New read calls:** a bare `read_to_string(`, `.open_with(`, `open_ambient_dir(`, `open_dir_nofollow(`, `.open_dir(`.
- **New write calls:** `hard_link(`, `symlink(`, `tempfile_in(`, `NamedTempFile::new_in(`, `tempdir_in(`, `persist_noclobber(`, `.open_with(`, `.rename(`, `.copy(`, `set_len(`.
- **Imports:** a `use std::fs::` import that names a free function is reported as a read or a write.
- **A third scan**, `no_spawned_git_command_writes_files`, reports a Git argument that writes files (`--output`, `worktree`, `checkout`, `restore`, `stash`, `clean`, `reset`, `archive`, `bundle`) unless it is marked `git writes no project file:` with the reason.
- **The hidden read** in `compute_coverage_checked` goes through `read_project_string` with the 8 MiB bound.
- **Wrong markers are corrected** in `hooks.rs` and `validate_loaded_change.rs`.
