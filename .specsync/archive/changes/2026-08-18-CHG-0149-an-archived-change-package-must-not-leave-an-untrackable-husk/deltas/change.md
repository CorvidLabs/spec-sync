## ADDED

### REQUIREMENT REQ-change-074

An archived change package SHALL NOT retain a directory that holds no regular file at any depth, and enumeration SHALL treat such a directory under the archive as an absent change rather than a damaged one.

Acceptance Criteria
- Shipping a change whose `deltas/` is empty leaves no untrackable directory in the dated archive package, so a checkout of a commit that predates the package removes the package entirely instead of stranding a husk that `git status` reports as clean.
- A directory under the archive that holds no regular file at any depth is skipped by `change new`, `change audit`, `change adopt` and `check`, since git cannot represent it and its presence records the absence of a change rather than a corrupt one.
- A directory under the archive that holds at least one regular file but no `state.json` is still refused, so the allowance cannot be satisfied by ignoring corruption.
- Directories in an archived package that do hold files are preserved, so pruning removes only what git could never have committed.
