---
id: the-upstream-archive-lookup-reads-tree-entries-as-bytes-and-knows-an-archive-head-already-holds-after-an-upstream
state: draft
type: bug_fix
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# The upstream archive lookup reads tree entries as bytes and knows an archive HEAD already holds after an upstream rename

## Intent

The upstream archive lookup reads tree entries as bytes and knows an archive HEAD already holds after an upstream rename

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- an unrelated archive package with a non-UTF-8 directory name on the remote default does not stop the lookup, and this change's upstream archive is still found; an archive HEAD already holds is not reported after the remote default renames its package; a non-UTF-8 package that does hold this change's record is still cannot-tell

## No-spec Rationale

REQ-change-129 refuses only an archive of the same change id that HEAD does not contain, and never one HEAD already contains; REQ-change-142 reports cannot-tell only when Git cannot list or read the records. An unrelated non-UTF-8 package name made the lookup cannot-tell (#961), and an upstream rename made an archive HEAD holds read as remote-only (#945)
