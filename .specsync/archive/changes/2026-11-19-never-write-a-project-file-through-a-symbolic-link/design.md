---
change: never-write-a-project-file-through-a-symbolic-link
artifact: design
---

# Design

**Shared functions in `src/util.rs`.** Each takes the project root and a path that is relative to it or absolute beneath it.

- `read_project_file(root, path) -> io::Result<Option<Vec<u8>>>`: `None` when nothing is there.
- `write_project_file(root, path, contents) -> io::Result<()>`: creates missing directories, replaces the content of an existing regular file, creates a missing one.
- `create_project_file(root, path, contents) -> io::Result<bool>`: `false` when a regular file is already there, otherwise an exclusive create.
- `remove_project_file(root, path) -> io::Result<()>`.
- `project_path_refusal(root, path) -> Option<String>`: the reason a path is refused, for a preflight.

All of them open a `cap_std::fs::Dir` on the root and act through it, so a path cannot leave the root even if a component is swapped while they run. A link inside the root is followed. Refused with a typed error whose message names the path: a link that leaves the root, an absolute link target, `..`, a dangling link, a target that is not a regular file.

**Callers.**

- `hooks`: the markdown and settings writers and `remove_section_from_file` take the root and use the functions. A refusal is the `Err` they already return.
- `agents`: the inspect read, the artifact write, the manifest read and write, and the uninstall removal use them. Every artifact is inspected before the first write, so a refusal comes before any write.
- `mcp`: `cmd_lock_write` uses `write_project_file`.
- `registry`: `register_module` returns `Result<bool, String>`; `scaffold` prints the error and exits 1.
- `cmd_resolve`: the cache checks its directories and file for any link, skips the entry with one warning, and writes through the same functions.
- `cmd_migrate`: a preflight calls `project_path_refusal` on every path the steps touch and on every discovered spec file, and stops before the first step. The steps then write through the functions.
- `generator`: the companion functions take the root and call `create_project_file`. A refused companion is reported on stderr and the rest continue.
- `cmd_scaffold`, `cmd_new`, `cmd_wizard`, `cmd_import`: the spec file is created with `create_project_file`.

**Not changed.** `hash_cache` keeps its own stricter handle from the first change. `hooks::is_installed` still only reads.
