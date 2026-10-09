---
change: name-clean-git-files-by-object-id-so-change-check-accepts-an-lfs-pointer
artifact: docs
---

# Docs

`docs/HLD.md` states the workspace digest domain as `specsync.project-input-digest.v3`. A clean tracked file is path, kind, mode, and Git object id. A dirty or untracked file is path, kind, mode, and bytes, inside the payload bound. `filter=lfs` is the pointer object.

The `change` spec contract item 23 and `REQ-change-103` say the same rule. Acceptance evidence still hashes covered path bytes.
