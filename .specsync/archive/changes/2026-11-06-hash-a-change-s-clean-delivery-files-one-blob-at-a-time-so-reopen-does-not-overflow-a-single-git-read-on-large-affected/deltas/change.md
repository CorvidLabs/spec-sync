## ADDED

### REQUIREMENT REQ-change-144

Building a change's acceptance manifest or acceptance input digest SHALL read the clean tracked files in its delivery scope one blob at a time, each under the existing per-object bound, so that `change reopen`, and every other read that builds that evidence, does not fail because the total size of those files exceeds a single in-memory Git read.

Acceptance Criteria
- Reopen of a stale accepted change whose clean affected files exceed one `git cat-file --batch` read, with or without an acceptance manifest, no longer fails with "`git cat-file --batch` output exceeds deterministic bounds".
- The acceptance manifest and the acceptance input digest are byte-identical to the ones the whole-scope read produced, for clean, modified, symlinked, deleted and executable files, LFS pointers and the sequence ledger, so every recorded manifest and digest still matches.
- Each clean blob is still bounded by `MAX_GIT_EVIDENCE_PAYLOAD_BYTES`; a blob over that bound is refused, and the refusal names it. The path-count bound and the Git read deadline are unchanged.
- The accepted-input validation behind `change status` and the archive preflight builds the same evidence the same way.
