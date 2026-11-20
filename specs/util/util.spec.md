---
module: util
version: 4
status: stable
files:
  - src/util.rs
  - src/util/local_state.rs
db_tables: []
depends_on: []
---

# Util

## Purpose

Provides small shared utility functions used by validators, fix suggestions, and rule compilation. The module keeps low-level helpers centralized so parsing and validation modules do not duplicate edit-distance or regex-safety logic.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `levenshtein` | `a: &str, b: &str` | `usize` | Computes Unicode-scalar edit distance between two strings |
| `safe_regex` | `pattern: &str` | `Option<regex::Regex>` | Compiles a user-provided regex with bounded regex and DFA size limits |
| `confine_path_to_root` | `root, rel` | `Option<PathBuf>` | Resolve a project-relative path and reject escapes outside the root |
| `read_project_file` | `root, path, max_bytes` | `io::Result<Option<Vec<u8>>>` | Read a file beneath the project root, `None` when nothing is there; follows a link that stays inside the root and refuses one that leaves it, a dangling link, a non-regular file and a file over `max_bytes`, opening without blocking and reading at most one byte past the bound (REQ-util-002, REQ-util-003) |
| `read_project_bytes` | `root, path, max_bytes` | `io::Result<Vec<u8>>` | `read_project_file` with the contract of `std::fs::read`: a missing file is a not-found error (REQ-util-003) |
| `read_project_string` | `root, path, max_bytes` | `io::Result<String>` | `read_project_file` with the contract of `std::fs::read_to_string`: a missing file is not found, bytes that are not UTF-8 are invalid data (REQ-util-003) |
| `MAX_STATE_FILE_BYTES` | constant | `u64` | 4 MiB: the bound for policy, config and state files and change records |
| `MAX_SPEC_FILE_BYTES` | constant | `u64` | 8 MiB: the bound for spec files, companions, history fragments, templates and hook files |
| `MAX_SOURCE_FILE_BYTES` | constant | `u64` | 8 MiB: the bound for source and schema files |
| `MAX_PROJECT_INPUT_BYTES` | constant | `u64` | 64 MiB: the bound for the transaction journal, the hash cache and other whole project inputs |
| `write_project_file` | `root, path, contents` | `io::Result<()>` | Write a file beneath the project root, creating it and its directories when missing; writes through a link that stays inside, refuses the rest and never creates through a link |
| `create_project_file` | `root, path, contents` | `io::Result<bool>` | Create a file that does not exist yet with an exclusive create; `false` and no change when a regular file is already there |
| `remove_project_file` | `root, path` | `io::Result<()>` | Remove a name beneath the project root; a link that stays inside is removed as a link |
| `create_project_dir` | `root, path` | `io::Result<()>` | Create a directory beneath the project root and the directories on the way |
| `project_path_refusal` | `root, path` | `Option<String>` | Why the project file functions refuse a path, naming it, without changing anything; `None` for a path they accept |
| `read_file_within` | `directory, path, max_bytes` | `io::Result<Vec<u8>>` | Read a file within exactly a directory the person names, by the same rule and bound, for a directory outside the project (REQ-util-004) |
| `create_new_project_dir` | `root, path` | `io::Result<()>` | Create a directory that must not exist yet, after the directories on the way; something already at the name is `AlreadyExists` |
| `is_project_file` | `root, path` | `bool` | Whether the path is a regular file the project file functions would read |
| `remove_empty_project_dir` | `root, path` | `io::Result<()>` | Remove an empty directory beneath the project root without following a link to it |
| `local_state` | module | `util::local_state` | Where SpecSync keeps its own state about a project, outside the work tree (REQ-util-006) |
| `git_state_location` | `canonical_root` | `Option<(PathBuf, PathBuf)>` | The Git directory of the work tree holding the root and the root's path in it, for the local state folder; `None` outside a work tree (REQ-util-006) |
| `work_tree_and_git_dir` | `root` | `Option<(PathBuf, PathBuf)>` | The canonical top folder of the work tree holding the root and its Git directory, from which the hooks folder is reached (REQ-hooks-009); `None` outside a work tree |
| `project_state_dir` | `root` | `io::Result<PathBuf>` | Where SpecSync keeps its own state about the project, outside the work tree: `<absolute-git-dir>/specsync/<key>` in a Git work tree, else `specsync/projects/<key>` in the user's cache folder (REQ-util-006) |
| `open` | `root, create` | `io::Result<Option<LocalState>>` | LocalState: the project's local state folder, made with `create`, opened without following a link at any name SpecSync made there; `None` when it does not exist yet |
| `file_path` | `&self, name` | `PathBuf` | LocalState: the path of a file in the folder, for messages |
| `subdirectory` | `&self, name, create` | `io::Result<Option<LocalState>>` | LocalState: a folder in this one, by the same rule |
| `metadata` | `&self, name` | `io::Result<Option<Metadata>>` | LocalState: what is at a name, without following a link |
| `read_file` | `&self, name, max_bytes` | `io::Result<Option<Vec<u8>>>` | LocalState: a file's bytes, opened without following a link or blocking, checked on the handle and bounded; `None` when missing |
| `write_file` | `&self, name, contents, durable` | `io::Result<()>` | LocalState: a new file renamed over the name, synced with the folder when `durable` |
| `remove` | `&self, name, durable` | `io::Result<()>` | LocalState: unlink a name; nothing there is not an error |
| `find_project_files` | `root, directory, suffix, max_entries, max_depth` | `io::Result<Vec<PathBuf>>` | Every file under a project directory whose name ends with `suffix`, sorted; walks through the project handle, lists file links for the reader to refuse, never walks a directory link or the Git directory, and stops past either budget (REQ-util-004) |

### Exported Types

| Type | Description |
|------|-------------|
| `LocalState` | A handle on a project's local state folder outside the work tree, and its path for messages (REQ-util-006) |
| `RefusedLocalState` | A local state location SpecSync will not use, with the reason, carried inside an `io::Error` |

## Invariants

1. `levenshtein` treats input as `char` sequences rather than raw bytes.
2. `levenshtein` returns `0` for equal strings and the other string's character length when one side is empty.
3. `safe_regex` never panics on invalid user input; invalid or oversized patterns return `None`.
4. `safe_regex` applies the same maximum limit to compiled regex size and DFA size.
5. `read_project_file`, `write_project_file`, `create_project_file`, `remove_project_file`, `create_project_dir` and the other project file functions act only through a `cap_std` directory handle, so nothing they touch is outside it, even if a path component is swapped while they run. Inside a Git work tree the handle is on the work tree's top folder (`git rev-parse --show-toplevel`, asked once per root with `GIT_DIR` and `GIT_WORK_TREE` unset) when the project root is that folder or Git tracks a file under the root, so a monorepo subproject follows a link that stays inside its repository; a project that is not part of the work tree around it, such as a ZIP unpacked in a home folder kept under Git, and any project outside a work tree, has the handle on the project root (REQ-util-007). Messages name paths relative to the project root. A symbolic link that stays inside is followed. A path that leaves through a link or `..`, a dangling link, and a target of the wrong kind are refused with an error naming the path, and an absolute link is never followed and is refused as an absolute link. A missing file is created with an exclusive create, never through a link (REQ-util-002, REQ-util-004).
6. `read_project_file`, `read_project_bytes` and `read_project_string` open the file through the project root handle without blocking, check on the open handle that it is a regular file, refuse it when it is larger than the bound the caller passes, and read at most one byte past that bound. A FIFO or a device therefore never holds a read, and no read grows without limit. The bounds are named constants: 4 MiB for state, 8 MiB for specs and source, 64 MiB for whole project inputs (REQ-util-003).
7. The project file functions refuse a path that resolves into the Git directory: a `.git` component in any letter case or with trailing dots or spaces, a path under the work tree's Git directory (`git rev-parse --absolute-git-dir`), or a link whose canonical target lies under either, with `<path> leads into the Git directory`. A link that stays inside the project therefore never reaches a Git hook or Git's config. SpecSync's own writes into the Git directory (`hooks install`) go through their own paths, not these functions (REQ-util-005).
8. `write_project_file` opens an existing file without blocking and without truncating, checks on the open handle that it is a regular file, and only then empties and writes it, so a FIFO swapped in after the check never holds the write (REQ-util-004).
9. `find_project_files` stops with the reason past `max_entries` entries or `max_depth` levels, so no tree a repository carries can hold a walk; a walk of the project root itself starts at the handle's own directory (REQ-util-004).
10. SpecSync's own state about a project, the hash cache, the transaction journal and the remote spec cache, is kept only in `project_state_dir`, never in the work tree, where a repository can put a file under any spelling a case-insensitive disk folds onto the real name, in an archive with no Git to say it came with the repository. The folder is keyed by a digest of the project root's path in the work tree, or of the canonical root, so two projects never share it. `LocalState` refuses a link at any folder name it makes, reads without blocking and within a bound, and writes by renaming a new file over the name, so it never writes through what a name was. Unit tests keep that folder in the system temporary directory (REQ-util-006).
11. The read and write scans in `tests/integration/project_reads.rs` and `project_writes.rs` scan exactly the source files Cargo's dep-info lists for the `specsync` binary, leave out only `#[cfg(test)]` items found by a lexer that ignores braces in comments and literals, and also report cap-std opens, renames, copies and whole-file reads and writes, temporary files, links, `set_len`, and free functions imported from `std::fs`. `no_spawned_git_command_writes_files_unmarked` reports a Git argument that writes files (`worktree`, `checkout`, `--output` ...) unless its statement says why no project file is written (REQ-util-008).

## Behavioral Examples

### Scenario: Suggest nearby filenames

- **Given** the strings `config.ts` and `confg.ts`
- **When** `levenshtein` compares them
- **Then** it returns `1`, allowing validation to suggest the near miss

### Scenario: Reject invalid regex

- **Given** an invalid pattern such as `[invalid`
- **When** `safe_regex` tries to compile it
- **Then** it returns `None` instead of propagating a regex parser error

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Empty string passed to `levenshtein` | Returns the character length of the other string |
| Invalid regex syntax | `safe_regex` returns `None` |
| Pattern exceeds configured regex size limits | `safe_regex` returns `None` |
| A project path leaves the root through a symbolic link or `..`, is a dangling link, or is not a regular file | The project file functions return an error naming the path and change nothing; `project_path_refusal` returns the same reason |
| A project file is a FIFO, a device, or larger than the bound passed | The read functions return an error naming the path and the reason (`is not a regular file`, `is larger than <n> bytes`) without waiting or reading past the bound |
| A project path is an absolute symbolic link, even one pointing inside | Refused: `<path> is an absolute link, which is never followed` |
| A project path, or a link's target, lies in the Git directory | Refused: `<path> leads into the Git directory`; nothing is written there |
| A walk passes its entry or depth budget | `find_project_files` returns an error naming the directory and the budget |
| A project inside another work tree that Git does not track there | The project file functions confine to the project root, so a link to a file of the outer work tree is refused as leaving the project |
| No Git work tree and no user cache folder (`HOME` and `XDG_CACHE_HOME` unset) | `project_state_dir` and `LocalState::open` return an error saying so; nothing is kept in the work tree instead |
| A symbolic link or a file where SpecSync makes its state folder, or a link, FIFO or oversized file at a name in it | Refused with the path and the reason; nothing is read or written through it |

## Dependencies

### Consumes

| Module | What is used |
|--------|--------------|
| regex | `RegexBuilder` for bounded regex compilation |

### Consumed By

| Module | What is used |
|--------|--------------|
| validator | `levenshtein`, `safe_regex` |
| parser | `levenshtein` |

## Change Log

| Date | Change |
|------|--------|
| 2026-10-10 | Add `util.3md` beside the spec. The first plane is context and later planes are lessons. `context.md` stays. |
| 2026-06-07 | Initial spec for shared utility helpers |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-08-01 | CHG-0071-land-pre-6-0-product-fixes-for-hooks-init-coverage-naming-and-exit-codes-scoped: Land pre-6.0 product fixes for hooks init coverage naming and exit codes (scoped paths) |
