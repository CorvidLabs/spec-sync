---
id: the-batched-object-reader-accepts-sha-256-object-ids
state: draft
type: bug_fix
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# The batched object reader accepts SHA-256 object ids

## Intent

The batched object reader accepts SHA-256 object ids

## Affected Canonical Specs

- `git_utils`

## Acceptance Criteria

- in a repository initialized with --object-format=sha256, show and ls-tree reads at a full 64-character commit id are answered by the one cat-file --batch child, byte for byte equal to per-file reads, and a missing path still reads as absent

## No-spec Rationale

REQ-git-utils-010 requires committed blob and tree reads at a fixed commit to be served by one cat-file --batch child; in a SHA-256 repository a read at a full 64-character commit id fell back to its own git show (#985)
