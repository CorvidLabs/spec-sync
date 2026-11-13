---
id: the-merge-driver-merges-appended-change-log-rows-after-a-stray-leading-row
state: draft
type: bug_fix
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# The merge driver merges appended Change Log rows after a stray leading row

## Intent

The merge driver merges appended Change Log rows after a stray leading row

## Affected Canonical Specs

- `merge`

## Acceptance Criteria

- Two branches that each bump version 168 to 169 and append a different Change Log row to a section that opens with a stray headerless row merge cleanly in either order with version 170, the stray row once and both new rows (red then green)

## No-spec Rationale

Makes the merge driver match REQ-merge-004 (MERGE-1): two branches that each append a change-log row merge in either order with both rows. A Change Log section opening with a stray headerless row made the driver read that row as the table header, so the row union never ran and the merge conflicted.
