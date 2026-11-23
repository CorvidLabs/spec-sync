---
change: keep-writes-reads-and-caches-a-repository-can-steer-inside-its-git-work-tree
artifact: design
---

# Design

## The confinement (`util`)

- **Where the handle sits.** `confinement(canonical_root)` asks `git rev-parse --show-toplevel --absolute-git-dir` once per root, with `GIT_DIR` and `GIT_WORK_TREE` removed, and caches the answer for the run.
  - Inside a work tree, every project path is opened beneath the top folder through one `cap_std::Dir`, and `prefix` is the root's path beneath it. Messages strip `prefix`, so they still name paths relative to the root.
  - Outside a work tree, or without Git, the top is the root itself.
- **`inspect_project_path`** walks the path one component at a time and refuses:
  - an absolute link target, as "is an absolute link, which is never followed";
  - a `.git` component in any letter case with trailing dots and spaces trimmed;
  - a path or a link's canonical target under the Git directory or a `.git` component, as "leads into the Git directory";
  - a link out, a dangling link, and anything that is not a regular file or directory, as before.
- **Writing an existing file** opens it non-blocking and without truncating, checks the handle is a regular file, then calls `set_len(0)` and writes (F9).
- **New helpers:**
  - `read_file_within(directory, path, max)` reads within exactly a directory the person names;
  - `create_new_project_dir` creates a directory that must not exist yet;
  - `is_project_file` says whether a path is a regular file the functions would read;
  - `remove_empty_project_dir` removes an empty directory without following a link;
  - `find_project_files(root, directory, suffix, max_entries, max_depth)` walks through the handle. It lists file links for the reader to refuse, does not walk directory links or the Git directory, and stops past either budget. When the directory is the root itself, it starts at the handle's own directory.

## Writers routed through the helpers

- **The change workflow (F2):**
  - `write_change_markdown`, `ensure_artifact_files` (exclusive create), `allocate_change_workspace` and `create_change` use the helpers. `write_record_json` replaces the production `write_json`.
  - `ensure_change_files_writable` refuses a change whose `state.json`, `change.md`, `approvals.json` or selected artifact is refused, before `answer`, `depend`, `supersede`, `approve` or `start` writes anything.
  - The durable transaction writers refuse every link component.
- **Other writers:**
  - `check --fix --backup` writes its backups and `--fix` writes its specs through the helpers (F3);
  - `init-registry` creates its file through them (F8);
  - `init`, `lifecycle`, `merge`, `generator`, `doctor`, `agents` and `retirement` write through them too.
- **Marked exceptions.** A write that is not a project file write carries a `not a project file write:` marker on its statement. Examples are a temporary directory, the Git hooks directory and the merge driver's `%A` file.

## Inputs a repository supplies

- **The hash cache (F4).** `tracked_cache_refusal` asks Git only when `.specsync/hashes.json` exists. If Git tracks it, `load` returns an empty cache, `save` refuses, and `check` prints `warning: hash cache not used:` with the `git rm --cached` fix. `prune` keeps only keys `confine_path_to_root` accepts that name a project file.
- **Configured directories (F5).** `validate_config_directories` runs in the ambient TOML and JSON loaders. The MCP snapshot check applies it per request. `check` and `issues` keep their own confined handling.
- **Spec discovery (F5).** `find_spec_files_checked` uses `find_project_files` with 100,000 entries and 256 levels. `find_spec_files` prints the reason and exits 1.
- **The changelog (F6).** `read_file_at_ref` asks `git cat-file -s` first and reads with `output_after_child_exits_capped`. A blob over the spec bound is skipped with one warning.
- **Imports (F7).** `read_import_file` reads a project file by the project rule. A file under an outside directory is read with `read_file_within` that directory.

## The scans (G1)

- **`no_project_file_is_written_outside_the_confined_helpers`** fails on a direct write, copy, create, rename, `persist`, removal or `set_permissions` outside the helpers. A statement-scoped marker excuses one.
- **The read scan** now also counts `fs::copy`, `.read(true)`, `BufReader::new` and `io::copy`.
- **Both scans** skip `#[cfg(test)]` blocks. Every remaining allow-listed file carries a reason.
