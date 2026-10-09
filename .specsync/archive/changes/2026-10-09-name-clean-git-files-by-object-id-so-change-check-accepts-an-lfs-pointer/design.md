---
change: name-clean-git-files-by-object-id-so-change-check-accepts-an-lfs-pointer
artifact: design
---

# Design

`PROJECT_DIGEST_DOMAIN` is `specsync.project-input-digest.v3`. `project_input_digest` calls discovered-evidence capture with `identify_clean_blobs` set. That flag is part of the evidence cache key.

When the flag is set, a clean regular file (`100644` or `100755`) contributes its object id and is left out of `git cat-file`. A clean symlink is still read so its target can be checked. Dirty files, untracked files, and non-Git reads still count against `MAX_GIT_EVIDENCE_PAYLOAD_BYTES` (256 MiB).

`validate_git_attribute_output` continues only for `filter=lfs`. `ident`, `working-tree-encoding`, and any other filter still fail closed.

Acceptance discovery, definition snapshots, and archive snapshots call the same capture with `identify_clean_blobs` clear, so they still hash bytes. `git_evidence_with_policy` and `git_worktree_state` do the same.

A stored v2 workspace digest will not match a v3 recompute. `change check` records the new digest. Archived changes are not re-hashed as the product check.
