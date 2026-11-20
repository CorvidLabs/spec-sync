## MODIFIED

### REQUIREMENT REQ-hash-cache-009

The hash cache SHALL be read and saved only as `hashes.json` in the project's local state folder outside the work tree (REQ-util-006). A `.specsync/hashes.json` in the work tree, in any letter case and whether or not Git tracks it, SHALL never be read or saved, so a cache that came with a repository (a clone, an unpacked archive, or a checkout where Git cannot answer) can never replay a stored pass. When no local state folder can be found, `load` SHALL return an empty cache, and `save` and `refusal` SHALL give the reason.

Acceptance Criteria
- A forged pass written to `.specsync/hashes.json` in a tree with no Git, and committed as `.specsync/Hashes.json` on a case-insensitive disk, is never replayed.
- `save` writes into the local state folder and leaves the work tree's `.specsync` unchanged.
