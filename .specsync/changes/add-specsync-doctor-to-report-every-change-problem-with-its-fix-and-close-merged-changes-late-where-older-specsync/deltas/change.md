## ADDED

### REQUIREMENT REQ-change-143

A late close SHALL move a change package that is committed on the remote default branch but was never archived, byte for byte, into `.specsync/archive/closed-late/<date>-<id>/` with a `late-close.json` record, without reading the change's delivery evidence and without adding a change state, and SpecSync SHALL never treat a closed-late change as verified.

Acceptance Criteria
- The package comes from `.specsync/changes/` or `.specsync/archive/changes/`. Its `state.json` keeps its bytes and the state in it, and every other file keeps its bytes.
- The package's `state.json`, at its current path, must be present on the remote default reference (`origin/HEAD`, else `origin/main` or `origin/master`). When no remote default resolves, or the package is not on it, the close is refused and nothing is written.
- No delivery input, acceptance manifest, acceptance input digest or verification is read or rebuilt.
- `late-close.json` records the schema version, change ID, actor, reason, timestamp, the state it was closed from, its workflow version, the path it came from, the remote default reference and its commit, the commit that added the package and that commit's date, and `verified: false`.
- For a member of an acknowledged sequence collision, the acknowledgement is removed from `.specsync/change-sequence.json` and recorded in `late-close.json`; the ledger's sequence and claim are unchanged.
- A closed-late change is never part of the archive preflight, the successor walk or the sequence accounting, which read only `.specsync/changes/` and `.specsync/archive/changes/`.
- `change status` (text and JSON) and `change ship-status` report a closed-late change as closed late, with who, when and why, the commit that merged it and that commit's date, never re-checked and not verified; one closed from draft is also reported as never approved.
- `reopen`, `review`, `check`, `accept`, `archive` and `finalize` refuse a closed-late change, and `change new` refuses its slug.
- SpecSync 6.0.0 and 6.2.0, run against a repository holding closed-late packages, pass `change audit`, omit them from `change list`, report them not found in `change status`, `change ship-status` and `change reopen`, and are not blocked by them in `change finalize` or `change archive`.
