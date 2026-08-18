# Requirements

## REQ-change-074 (new)

An archived change package SHALL NOT retain a directory that holds no regular file at any
depth, and enumeration SHALL treat such a directory under the archive as an absent change
rather than a damaged one.

See `deltas/change.md` for the canonical delta applied to `specs/change/requirements.md`.

## Requirements deliberately not changed

`REQ-change-033` (declared path ownership) and the archive-integrity requirements are
untouched. This change narrows *what counts as an archive package*, not what a package must
contain once it is one.
