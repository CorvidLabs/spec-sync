## ADDED

### REQUIREMENT REQ-hash-cache-009

A `.specsync/hashes.json` that Git tracks, in the index or in `HEAD`, came with the repository and SHALL never be read or saved over: `load` SHALL return an empty cache, `save` SHALL refuse, and `refusal` SHALL give the reason with the `git rm --cached .specsync/hashes.json` fix. Every digest in a cache is over public inputs, so a committed cache could otherwise carry a pass for a spec that fails. Git SHALL be asked only when a cache file is present.

Acceptance Criteria
- A cache carrying a forged pass replays while untracked and not once Git tracks it; `save` then refuses and the tracked bytes stay.
- `rehash` exits 1 on a tracked cache and leaves it unchanged.

### REQUIREMENT REQ-hash-cache-010

`prune` SHALL keep an entry only for a regular file the project rule reaches, and SHALL drop a key that is absolute or holds `..` without looking at the path it names.

Acceptance Criteria
- An absolute key and a `..` key naming an existing outside file are dropped; a key for a project file is kept.
