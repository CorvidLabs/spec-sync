---
change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
artifact: research
---

# Research

Measured on Git 2.54 with a blobless clone whose remote resolves only through the user's global `url.<x>.insteadOf` (the stand-in for a credential helper the isolation drops): the isolated `git show <old commit>:<path>` fails with `could not fetch <oid> from promisor remote`; the isolated `git rev-list --objects --missing=print HEAD -- <paths>` lists the missing objects without fetching; one `git fetch origin --no-tags --no-write-fetch-head --filter=blob:none --stdin`, run with the user's configuration and the ids on stdin, downloads them; the isolated read then succeeds and nothing is missing.
