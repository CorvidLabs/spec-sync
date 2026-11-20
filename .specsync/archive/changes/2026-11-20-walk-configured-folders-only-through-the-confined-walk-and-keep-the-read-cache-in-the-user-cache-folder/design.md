---
change: walk-configured-folders-only-through-the-confined-walk-and-keep-the-read-cache-in-the-user-cache-folder
artifact: design
---

# Design

- **`util::read_project_dir(root, path)`** is a one-level listing through the project rule. It is used by the schema snapshot, `check`'s schema inputs, `scaffold`'s case-collision check (`check_case_collision` now takes the root) and `suggest_similar_file`.
- **`validator::find_files_named(root, dir, name)`** is the confined, budgeted walk for exact names. It is used by `change adopt`'s requirement proposals and the duplicate-ID check.
- **`find_spec_files_checked`** is used by `generate_registry` and the MCP issues walk.
- **`find_project_files`** is used by the wizard's source walk.
- **Source auto-detection** skips a child of the root that is a symbolic link, and walks with `follow_root_links(false)`.
- **`local_state::repository_state_dir(root, common_dir)`** gives `<user cache>/specsync/repositories/<key>`, where the key is a digest of the canonical common directory. A cache folder inside the project is refused, as for the other state. `persistent_read_cache_path` returns that folder joined with `read-cache-v1.json`.
