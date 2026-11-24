---
change: pass-safe-directory-to-specsync-s-isolated-git-when-the-user-s-own-git-trusts-a-repository-owned-by-another-user
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-change-171 | `tests/integration/git_environments.rs`, with `GIT_TEST_ASSUME_DIFFERENT_OWNER=1`: a repository the user's global file trusts passes `change audit`, `change status` and `change check`, also from a linked worktree; without the entry it is refused naming dubious ownership, the path and the command; `safe.directory = *` passes only the top level; the file is outside the project and gone afterwards; a repository the user owns keeps the global configuration off |
