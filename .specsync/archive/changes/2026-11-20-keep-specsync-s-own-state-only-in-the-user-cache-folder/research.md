---
change: keep-specsync-s-own-state-only-in-the-user-cache-folder
artifact: research
---

# Research

- **What a clone can and cannot bring.** `git clone` never fills the Git directory from the remote. A tarball of a working copy carries all of `.git`, though, and a `.git` file can point anywhere inside the archive. Nothing SpecSync can ask tells these apart.
- **The user cache folder.** No repository content lands there, unless `HOME` or `XDG_CACHE_HOME` points into the project, which some CI containers do. A base inside the project, or inside the folder it is confined to, is refused.
- **Git pathspecs.** `*`, `?` and `[...]` are wildcards unless `--literal-pathspecs` (or `GIT_LITERAL_PATHSPECS=1`) is given, and `*` also matches `/`. The `HEAD:./path` revision lookup is already literal. `GIT_GLOB_PATHSPECS` and `GIT_ICASE_PATHSPECS` in the environment change matching too, so they are cleared.
- **`git config --show-scope --get core.hooksPath`** prints the scope (`global`, `system`, `local`, `worktree`, `command`) and the value.
- **Counts in the compiled sources at `e3d508ac`:**
  - `open_ambient_dir(`: 44 sites, most in exempt files.
  - `File::open_ambient(`: 0.
  - `use std::fs as`: 0.
  - `read_dir(`: 55 sites in 25 files; `WalkDir::new(`: 16. These are not small, so they are left out.
