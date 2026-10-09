---
change: name-clean-git-files-by-object-id-so-change-check-accepts-an-lfs-pointer
artifact: context
---

# Context

`change check` records one workspace digest over every Git-visible project file. Published 6.0.0 rejects any Git content filter, including `filter=lfs`, and a pointer-aware build still loads every clean blob and sums those bytes against the 256 MiB payload bound.

A clean tracked file is already a Git object. The workspace digest names that object id. It reads bytes only for dirty or untracked paths, and those reads stay inside the existing bound. `filter=lfs` is the pointer object. Every other content filter still fails closed. A change's own acceptance evidence still hashes the bytes of the paths it covers.

The digest domain is `specsync.project-input-digest.v3`. A recorded v2 digest does not match a v3 recompute, so an in-flight verification is stale until `change check` records a new one. Archives stay history.

Peck keeps LFS enabled and does not edit its ledger. This change does not raise the 256 MiB cap.
