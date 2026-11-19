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

