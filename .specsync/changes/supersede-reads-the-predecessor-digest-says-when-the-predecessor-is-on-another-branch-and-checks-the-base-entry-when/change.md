---
id: supersede-reads-the-predecessor-digest-says-when-the-predecessor-is-on-another-branch-and-checks-the-base-entry-when
state: draft
type: feature
base_commit: bb7682a78033aa06146baa638c44b241b6b937e1
---

# Supersede reads the predecessor digest, says when the predecessor is on another branch, and checks the base entry when declared

## Intent

Supersede reads the predecessor digest, says when the predecessor is on another branch, and checks the base entry when declared

## Affected Canonical Specs

- `change`
- `cmd_change`
- `cli_args`

## Acceptance Criteria

- supersede without --digest records the predecessor's signed entry digest for the path; a predecessor no change here records is refused as not in this branch's history with the merge or rebase step; when the predecessor's entry no longer matches the successor's base, supersede still records the obligation and warns, naming the commit that changed the file and the reconcile-history alternative

## No-spec Rationale

Not applicable
