---
change: name-clean-git-files-by-object-id-so-change-check-accepts-an-lfs-pointer
artifact: plan
---

# Plan

1. Let `project_input_digest` name each clean regular blob by its Git object id. Do not `git cat-file` those blobs. Keep the domain `specsync.project-input-digest.v3`.
2. Accept only `filter=lfs` in canonical Git attribute checks. Keep every other content filter fail-closed.
3. Leave acceptance, definition, and archive snapshots hashing real bytes.
4. Keep dirty and untracked reads inside the existing 256 MiB payload bound.
5. Record the rule on the `change` contract and on `REQ-change-103`, and name it in `docs/HLD.md`.
