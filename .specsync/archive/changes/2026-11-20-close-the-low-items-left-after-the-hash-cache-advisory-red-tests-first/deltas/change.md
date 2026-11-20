## MODIFIED
### REQUIREMENT REQ-change-162

The change workflow SHALL write, replay and remove its transaction journal only as `change-transaction.json` in the project's folder in the user's state folder (REQ-util-006), and only into files the change workflow writes.
- Each journal SHALL record the device and inode of the project root and its canonical path when it is written. A journal that records another root, or none (every journal SpecSync 6.4.0 wrote, in the user's cache folder or the state folder), SHALL never be replayed: every command that takes the lifecycle lock SHALL refuse, name the journal's path, say that it was not written for this project, and say how to clear it (check the files it lists, then delete it).
- A `.specsync/change-transaction.json` in the work tree, of any kind and whether or not Git tracks it, SHALL never be opened or replayed, and nor SHALL a journal under a Git directory. While one is in the work tree, every command that takes the lifecycle lock SHALL refuse, name the path, say that a journal in the work tree is never replayed because a repository can put one there, and say how to clear it (check the files it lists if an older SpecSync left it, then delete it).
- A journal entry SHALL be replayed only when its path is under `.specsync/`, or is a spec file, `requirements.md` or `history/*.md` under the configured `specs_dir`, or is a registry-mapped spec whose mapped path ends in `.spec.md`, the `requirements.md` beside it, or a file in that spec's `history/` directory.
- A path with a `.git` component SHALL be refused under ASCII case folding, after trailing dots and spaces are removed from each component.
- One refused entry SHALL refuse the whole journal, so nothing is replayed, with the existing invalid-journal message and its `.broken` recovery naming the entry.
- The bare-array journal form, written only by SpecSync 5.x, SHALL no longer be replayed and SHALL be refused the same way.
- Every transaction this build writes SHALL pass the same path rule before it is journaled.

Acceptance Criteria
- A valid journal planted at `.specsync/change-transaction.json`, or under `.git/specsync/`, is not replayed.
- A registry entry mapping a module to `.env` does not let a journal delete or rewrite `.env`.
- While a transaction runs, its journal is in the local state folder and not in the work tree.
- A journal left for an earlier project at the same path is not replayed, and the command names it.
- A journal SpecSync 6.4.0 left in the user's cache folder is not replayed, and the command names it.
