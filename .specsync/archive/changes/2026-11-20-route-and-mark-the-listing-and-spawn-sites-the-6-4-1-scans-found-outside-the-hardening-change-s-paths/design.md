---
change: route-and-mark-the-listing-and-spawn-sites-the-6-4-1-scans-found-outside-the-hardening-change-s-paths
artifact: design
---

# Design

- **Record folders** (doctor's archive and closed-late listings, `spec_history`'s `history/`) are listed through `util::read_project_dir_entries`.
- **`agents`, `config`, `migrate` and the MCP preflight** list project folders through it too.
- **Every other listing or walk** keeps its code and carries a `not a project file read:` marker with its reason: walks with `follow_links(false)`, the confined handle in `consumed_by_re`, the `--from-dir` folder in `import`, and the archived package SpecSync just moved.
- **Spawned `gh`, `date` and the `specsync` rerun in `watch`** carry `spawned command writes no project file:`. `merge`'s Git `config` calls carry `git writes no project file:`.
- **`generator`** reads `--template` files through `read_typed_file`, and `check_template_dir` is documented in its spec.
