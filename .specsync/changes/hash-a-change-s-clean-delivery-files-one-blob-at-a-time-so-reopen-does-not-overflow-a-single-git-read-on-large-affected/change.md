---
id: hash-a-change-s-clean-delivery-files-one-blob-at-a-time-so-reopen-does-not-overflow-a-single-git-read-on-large-affected
state: draft
type: bug_fix
base_commit: f7180c3481814c7b05671fb413b1ffec5de829e2
---

# Hash a change's clean delivery files one blob at a time so reopen does not overflow a single Git read on large affected paths

## Intent

Hash a change's clean delivery files one blob at a time so reopen does not overflow a single Git read on large affected paths

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- SHIP-6 (REQ-change-144): reopen of a stale accepted change whose clean affected files exceed one git cat-file --batch read, with or without an acceptance manifest, no longer fails with "`git cat-file --batch` output exceeds deterministic bounds"; a test-only seam shrinks the cap so the test runs on small fixtures.
- The acceptance manifest and the acceptance input digest are byte-identical to the ones the whole-scope read produced, for clean, modified, symlinked, deleted and executable files, an LFS pointer and the sequence ledger, so every recorded manifest and digest still matches.
- Each clean blob is still bounded by MAX_GIT_EVIDENCE_PAYLOAD_BYTES, and a blob over it is refused by name; the path-count bound and the Git read deadline are unchanged, and every other caller of inspect_git_candidates keeps both of its caps.
- The accepted-input validation behind change status and the archive preflight builds that evidence the same way.
- Failing tests first: each new test fails on main f7180c34 and passes after the change; no gate, timeout or assertion is loosened.

## No-spec Rationale

Not applicable
