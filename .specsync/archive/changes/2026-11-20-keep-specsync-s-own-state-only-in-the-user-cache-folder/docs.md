---
change: keep-specsync-s-own-state-only-in-the-user-cache-folder
artifact: docs
---

# Docs

- **`CHANGELOG.md`:** the change 6 entry says the state lives only in the user cache folder, so an archive that ships its `.git` cannot carry it. It also says:
  - a cache folder inside the project is refused;
  - F5 reads a FIFO, which blocks;
  - in CI on a reused runner, run `check --force`.
- **`site/src/content/docs/cli.md` and `docs/HLD.md`:** the location.
- **Specs:**
  - `util`: `project_state_dir`, invariant 10, the error rows, and the removal of `git_state_location`;
  - `git_utils`: `tracked_by_git`;
  - `change`: the replay rule;
  - `hooks`: invariant 16;
  - `cmd_check`: the no-folder row.

  Testing and tasks are updated, the stale `git_utils/testing.md` row is fixed, and a lesson is added to `util.3md`.
- **The private advisory additions:** the location, and R5b as documented.
