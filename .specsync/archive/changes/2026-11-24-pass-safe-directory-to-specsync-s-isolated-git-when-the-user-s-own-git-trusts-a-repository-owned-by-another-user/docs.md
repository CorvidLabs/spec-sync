---
change: pass-safe-directory-to-specsync-s-isolated-git-when-the-user-s-own-git-trusts-a-repository-owned-by-another-user
artifact: docs
---

# Docs

`site/src/content/docs/` says that SpecSync reads a repository owned by another user when the user's own Git trusts it (`safe.directory`), and names the command when it does not.
