---
change: name-clean-git-files-by-object-id-so-change-check-accepts-an-lfs-pointer
artifact: requirements
---

# Requirements

### REQ-change-103

The workspace digest SHALL name each clean tracked file by its Git object id. It SHALL read file bytes only for dirty or untracked paths, and those reads SHALL stay inside the existing payload bound. `filter=lfs` SHALL be accepted as that object id. Every other content filter SHALL still fail closed. Acceptance evidence for a change SHALL still hash the bytes of the paths that change covers.

#### Acceptance Criteria

- A clean `filter=lfs` file does not fail the workspace digest, and the recorded identity is the Git object id rather than the working-tree bytes.
- A dirty copy of that file changes the workspace digest.
- `filter=demo`, `working-tree-encoding`, and `ident` still fail closed.
- Scoped acceptance of a covered path still hashes that path's bytes.
- The 256 MiB bound still applies to bytes that are read.
