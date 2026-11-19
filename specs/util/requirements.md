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

