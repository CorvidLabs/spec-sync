---
id: support-git-lfs-pointers-and-hydrated-assets-in-canonical-change-evidence-without-hiding-content-changes
state: implementing
type: bug_fix
base_commit: 6a47f2cd0c6dc5dfa4168b79ac6dda960966bcfc
---

# Support Git LFS pointers and hydrated assets in canonical change evidence without hiding content changes

## Intent

Support Git LFS pointers and hydrated assets in canonical change evidence without hiding content changes

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- I can complete verification in a repository containing Git LFS assets. I can trust verification to detect changed asset content, whether assets are downloaded or represented by LFS pointers. Preserve whole-workspace coverage and fail closed for unsupported filters.

## No-spec Rationale

Not applicable
