---
id: an-archive-introduction-listing-over-the-output-bound-is-split-instead-of-read-one-commit-at-a-time
state: draft
type: bug_fix
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# An archive-introduction listing over the output bound is split instead of read one commit at a time

## Intent

An archive-introduction listing over the output bound is split instead of read one commit at a time

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- with an output bound below the combined listing, archive_introduction_listings splits it in halves until each part fits, returns listings equal to the unbounded ones, and spawns fewer Git processes than archive-introducing commits; a single commit over the bound is still read on its own

## No-spec Rationale

REQ-change-135 says the Git processes a cold archived change status spawns SHALL NOT grow with the number of commits that introduced archives; a combined diff-tree listing over the output bound fell back to one diff-tree per commit (#942)
