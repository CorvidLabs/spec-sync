---
change: keep-specsync-s-own-state-only-in-the-user-cache-folder
artifact: design
---

# Design

- **`local_state::location`** always gives `<user cache>/specsync/projects/<sha256("project\0" + canonical root)[..16]>`.
  - `util::git_state_location` is removed.
  - The cache base is canonicalized, or its nearest existing ancestor is. A base inside the canonical root, or inside `util::confinement_top` (the folder the project file functions confine to), is refused with `RefusedLocalState` and the reason.
  - Old `.git/specsync/` state is never read or removed.
- **`git_utils::tracked_by_git`** runs `git --literal-pathspecs ls-files --error-unmatch -- <path>` and `git cat-file -e HEAD:./<path>`, with `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE`, `GIT_LITERAL_PATHSPECS`, `GIT_GLOB_PATHSPECS`, `GIT_NOGLOB_PATHSPECS` and `GIT_ICASE_PATHSPECS` removed.
- **`replayable_transaction_path`** accepts a registry entry's spec, its `requirements.md` and `history/` only when the mapped path ends in `.spec.md`.
- **`hooks::git_hooks_dir`.** When the hooks path lies outside both the work tree and the Git directory, `git config --show-scope --get core.hooksPath` is asked.
  - With `global` or `system`, missing folders may be made, as before.
  - Otherwise nothing missing is made. A missing folder is an error, and an existing one is used.
- **The scans** add `open_ambient_dir(`, `File::open_ambient(`, `.open(`, and `use std::fs as` / `std::fs::{self as` aliases. Each new finding is marked with its reason.
