## MODIFIED

### REQUIREMENT REQ-change-162

The change workflow SHALL write, replay and remove its transaction journal only as `change-transaction.json` in the project's local state folder outside the work tree (REQ-util-006), and only into files the change workflow writes.
- A `.specsync/change-transaction.json` in the work tree, of any kind and whether or not Git tracks it, SHALL never be opened or replayed. While one is there, every command that takes the lifecycle lock SHALL refuse, name the path, say that a journal in the work tree is never replayed because a repository can put one there, and say how to clear it (check the files it lists if an older SpecSync left it, then delete it).
- A journal entry SHALL be replayed only when its path is under `.specsync/`, under the configured `specs_dir`, or is a spec file the local registry maps, the `requirements.md` beside it, or a file in that spec's `history/` directory.
- A path with a `.git` component SHALL be refused under ASCII case folding, after trailing dots and spaces are removed from each component.
- One refused entry SHALL refuse the whole journal, so nothing is replayed, with the existing invalid-journal message and its `.broken` recovery naming the entry.
- The bare-array journal form, written only by SpecSync 5.x, SHALL no longer be replayed and SHALL be refused the same way.
- Every transaction this build writes SHALL pass the same path rule before it is journaled.

Acceptance Criteria
- A valid journal planted at `.specsync/change-transaction.json` in a tree with no Git is not replayed: the file it would remove stays, and the command refuses naming the path.
- While a transaction runs, its journal is in the local state folder and not in the work tree.
