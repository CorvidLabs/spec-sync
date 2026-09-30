---
change: support-git-lfs-pointers-and-hydrated-assets-in-canonical-change-evidence-without-hiding-content-changes
artifact: requirements
---

# Requirements

### REQ-change-103

Canonical evidence SHALL support ordinary SHA-256 Git LFS v1 files without dropping them from evidence or invoking LFS filters.

Acceptance Criteria
- A valid pointer and its hydrated content have the same canonical payload and digest, including in-scope assets.
- Modified content, pointer targets, executable modes and deletion remain detectable.
- Hydrated content is streamed with bounded memory; it cannot exceed the aggregate evidence buffer by its raw size.
- Unsupported pointer extensions, malformed pointer-like inputs and other Git content filters remain errors.
- No LFS download or external clean/smudge/process filter is needed to collect evidence.
- Existing sparse, topology, exact attribute-output and stable capture checks remain in force.

