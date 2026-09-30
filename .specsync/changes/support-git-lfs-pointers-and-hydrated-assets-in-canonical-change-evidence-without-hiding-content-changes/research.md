---
change: support-git-lfs-pointers-and-hydrated-assets-in-canonical-change-evidence-without-hiding-content-changes
artifact: research
---

# Research

inspect_git_candidates validates attributes before substituting clean index blobs. It currently rejects filter=lfs, and dirty hydrated content would otherwise be buffered wholesale. Git LFS v1 specifies SHA-256, byte size and a bounded pointer; empty files pass through unchanged. Keep all governed paths rather than reinterpret ignored_paths.
