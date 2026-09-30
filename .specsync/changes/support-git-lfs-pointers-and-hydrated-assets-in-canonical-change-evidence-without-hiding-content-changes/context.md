---
change: support-git-lfs-pointers-and-hydrated-assets-in-canonical-change-evidence-without-hiding-content-changes
artifact: context
---

# Context

Fix CorvidLabs/spec-sync#789. The current whole-workspace evidence collector rejects unchanged LFS assets even outside source scope and ignored_paths. Bruno confirmed EVIDENCE-1 and EVIDENCE-2 before implementation.
