---
change: CHG-0081-make-a-fresh-project-usable-out-of-the-box-stop-a-leftover-directory-from-block
artifact: docs
---

# Docs

User-visible message changes only; no CLI surface, flag or file-format change.

- `init` emits a warning naming .specsync/sdd.json and an example command when no
  test command is detected.
- The "failed to read active change state" error no longer fires for a directory
  left behind by `git checkout`.

No README, MIGRATION or site page describes the previous behaviour, so no doc
edits are required.
