---
change: close-the-low-items-left-after-the-hash-cache-advisory-red-tests-first
artifact: design
---

# Design

- **State folders (S5, N2, L2, S2).**
  - `local_state` gains the user's state folder for the journal, beside the cache folder for the
    caches.
  - `location` refuses a project folder that would lie inside the project (N2).
  - On Unix, opening `specsync/…` checks each folder's owner and mode on the handle (L2).
  - The journal envelope gains `root` (device, inode, canonical path). Replay refuses a journal
    whose root differs or is missing, naming the file. A 6.4.0 journal in the cache folder is
    refused the same way.
- **The read cache (C1)** goes through `LocalState`'s read and write.
- **Hooks (S1, X2, H1).**
  - `reach_git_hooks_dir` adds the common Git directory as a base.
  - It compares resolved paths.
  - It refuses any outside folder unless the person's own config set it.
  - `rev-parse` runs with a clean environment.
- **Listings (W1, X1).**
  - The record-folder listings go through `read_project_dir`.
  - The read scan adds `read_dir(` and `WalkDir::new(`.
  - The Git scan adds `apply`, `merge`, `switch` and `config`.
  - A new scan reports non-Git spawns.
  - Sites that list SpecSync's own state folders, or read Git's config, are marked with the
    reason.
- **Typed paths (S3):** a new `util::read_typed_file` (no-block open, type check, bound), used by
  `--manifest` and `--template`.
- **Refused spec reads (C2 to C4):** `filter_by_status` keeps a spec whose frontmatter read is
  refused; `resolve` prints the refusal; `stale` adds it to the unmeasurable list.
- **Cache bound (C5):** `HashCache::save` refuses JSON over `MAX_CACHE_BYTES` before writing.
