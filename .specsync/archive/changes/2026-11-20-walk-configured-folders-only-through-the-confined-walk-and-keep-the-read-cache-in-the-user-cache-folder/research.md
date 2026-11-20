---
change: walk-configured-folders-only-through-the-confined-walk-and-keep-the-read-cache-in-the-user-cache-folder
artifact: research
---

# Research

- **`walkdir`** follows a link at the walk's root by default (`follow_root_links`), even with `follow_links(false)`.
- **`read_dir`** follows every component of its path.
- **`util::find_project_files`** walks through the project handle and refuses a folder that leaves the project. It never walks a directory link, skips the Git directory, and stops past its entry and depth budgets.
- **The read cache** is shared by worktrees today because it lives in the common directory. Keying its user-cache folder by the canonical common directory keeps that sharing.
