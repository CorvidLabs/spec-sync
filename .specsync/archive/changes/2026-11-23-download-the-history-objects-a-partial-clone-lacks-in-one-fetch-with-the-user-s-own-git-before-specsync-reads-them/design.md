---
change: download-the-history-objects-a-partial-clone-lacks-in-one-fetch-with-the-user-s-own-git-before-specsync-reads-them
artifact: design
---

# Design

- **Detection:** `git config --get-regexp ^(extensions\.partialclone|remote\..*\.promisor)$` names the promisor remote; none means a full clone and nothing more happens.
- **Listing, isolated:** `git rev-list --objects --missing=print HEAD [<default branch>] -- .specsync <requirements paths>`; it never fetches.
- **Fetch, once per process:** `git -C <root> fetch <remote> --no-tags --no-write-fetch-head --filter=blob:none --stdin` with the ids on stdin, the user's environment and configuration, `GIT_TERMINAL_PROMPT=0`, a new session with no controlling terminal, killed after 60 s. One line on standard error first: `downloading <n> objects this partial clone lacks from \`<remote>\``.
- **Refusal:** when the fetch fails, `<n> history objects are not in this partial clone, and SpecSync could not download them from \`<remote>\`: <reason>. Download them with your own Git: \`git -C <root> fetch <remote> --no-tags --no-write-fetch-head --filter=blob:none <oid> …\`, then run the command again.`
