---
id: pass-safe-directory-to-specsync-s-isolated-git-when-the-user-s-own-git-trusts-a-repository-owned-by-another-user
state: archived
type: bug_fix
base_commit: c076583c09ff9acbe529426922a3e225eed42f56
---

# Pass safe.directory to SpecSync's isolated Git when the user's own Git trusts a repository owned by another user

## Intent

Pass safe.directory to SpecSync's isolated Git when the user's own Git trusts a repository owned by another user

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- (1) A repository owned by another user, which the user's global configuration lists in safe.directory, passes change audit, change status and change check as for its owner, including from a linked worktree. (2) Without that entry it is refused naming dubious ownership, the path and git config --global --add safe.directory. (3) A repository owned by the user keeps the global configuration off, as before. (4) safe.directory = * passes the repository's own top level, never *. (5) The file is owner-only, outside the project and its Git directory, and gone when the command ends.

## No-spec Rationale

Not applicable
