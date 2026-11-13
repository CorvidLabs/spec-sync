---
id: the-delivery-blob-deadline-bounds-each-git-read-not-the-manifest-work-between-reads
state: draft
type: bug_fix
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# The delivery blob deadline bounds each Git read, not the manifest work between reads

## Intent

The delivery blob deadline bounds each Git read, not the manifest work between reads

## Affected Canonical Specs

- `change`
- `git_utils`

## Acceptance Criteria

- a delivery blob stream whose caller spends longer than the Git deadline between two reads still reads the second blob; a read that Git itself stalls past the deadline is still killed and reported as a deadline failure

## No-spec Rationale

REQ-change-144 keeps the Git read deadline unchanged, and the whole-scope read it replaced bounded only Git's own I/O; the delivery blob stream armed one deadline for its whole life, so the acceptance manifest's config, owner and manifest work between reads counted against Git (#991)
