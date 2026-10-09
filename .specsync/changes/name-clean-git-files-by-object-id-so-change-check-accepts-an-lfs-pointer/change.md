---
id: name-clean-git-files-by-object-id-so-change-check-accepts-an-lfs-pointer
state: implementing
type: feature
base_commit: 6a47f2cd0c6dc5dfa4168b79ac6dda960966bcfc
---

# Name clean Git files by object id so change check accepts an LFS pointer

## Intent

Name clean Git files by object id so change check accepts an LFS pointer

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- A clean filter=lfs file is named by its Git object id and is not read from the worktree. A dirty copy of that file changes the workspace digest. Other content filters still fail. Scoped acceptance still hashes the bytes of the paths the change covers. The 256 MiB bound still applies to bytes that are read.

## No-spec Rationale

Not applicable
