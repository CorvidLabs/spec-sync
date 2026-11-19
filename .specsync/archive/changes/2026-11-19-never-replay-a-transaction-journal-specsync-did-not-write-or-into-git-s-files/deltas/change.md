## ADDED

### REQUIREMENT REQ-change-162

The change workflow SHALL replay `.specsync/change-transaction.json` only when SpecSync on this machine can have written it, and only into files the change workflow writes.
- A journal that Git tracks, in the index or in `HEAD`, SHALL never be replayed. The writing command SHALL refuse, name the journal, say that it came with the repository, and give the fix (`git rm --cached .specsync/change-transaction.json`, then delete it).
- A journal entry SHALL be replayed only when its path is under `.specsync/`, under the configured `specs_dir`, or is a spec file the local registry maps, the `requirements.md` beside it, or a file in that spec's `history/` directory.
- A path with a `.git` component SHALL be refused under ASCII case folding, after trailing dots and spaces are removed from each component.
- One refused entry SHALL refuse the whole journal, so nothing is replayed, with the existing invalid-journal message and its `.broken` recovery naming the entry.
- The bare-array journal form, written only by SpecSync 5.x, SHALL no longer be replayed and SHALL be refused the same way.
- Every transaction this build writes SHALL pass the same path rule before it is journaled.

Acceptance Criteria
- A journal entry for `.git/config`, `.GIT/config`, `.git./config`, `.git/HEAD` (as a deletion) or `.git/hooks/pre-commit` is refused, and `.git/config`, `.git/HEAD` and the hook keep their bytes.
- A journal Git tracks, in the index or only in `HEAD`, is refused even when every entry names an allowed file, and the allowed file keeps its bytes.
- A journal with one allowed entry and one entry for `Makefile` replays neither.
- A bare-array journal is refused with the `.broken` recovery and replays nothing.
- An untracked durable journal for a spec, its `requirements.md`, a history fragment and a change record is still replayed, and so is one for a registry-mapped spec outside `specs_dir`. A file beside that spec that the workflow does not write is refused.
- `change new` with a committed journal that targets `.git/config` leaves `.git/config` unchanged.
