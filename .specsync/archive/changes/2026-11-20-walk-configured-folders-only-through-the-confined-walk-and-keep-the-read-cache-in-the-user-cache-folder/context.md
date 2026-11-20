---
change: walk-configured-folders-only-through-the-confined-walk-and-keep-the-read-cache-in-the-user-cache-folder
artifact: context
---

# Context

While change 7 was being built, two more gaps of the same classes were found and put to Leif by t4c. He answered "continue with 95% confidence" (2026-11-20, about 04:09Z), which t4c read as yes to both. Under Leif's delegation of 2026-11-20, t4c decided they go into a change 8 on top of change 7, with no send-back.

- **Configured-folder walks.** About nine walks start with `walkdir` or `read_dir` at a configured folder, and `walkdir` follows a link at its root. So a committed `specs`, source or schema link that leaves the project is walked:
  - `change adopt`'s requirement proposals counted two `requirements.md` files outside the project;
  - the duplicate requirement ID check;
  - `init-registry`'s registry;
  - the MCP issues walk;
  - the wizard's source walk;
  - the schema snapshot and `check`'s schema inputs;
  - `scaffold`'s case-collision listing;
  - a missing file's suggestion;
  - source auto-detection.

  File reads were already bounded and refused links out, but the walks themselves could run over the disk and show names or counts.
- **The read cache.** `src/change/read_cache.rs` keeps passing verdicts for archived change history in the Git common directory. An archive that ships `.git` could plant them.
