---
id: approvals-forked-across-clones-are-refused-until-one-fresh-approval
state: draft
type: feature
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# Approvals forked across clones are refused until one fresh approval

## Intent

Approvals forked across clones are refused until one fresh approval

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- a change approved separately on two branches that then merge is refused by definition-approval validation (check, audit, status) naming both approvers and the fresh approval that settles it; one fresh change approve on the merged branch settles it; approvals that only append are not a fork

## No-spec Rationale

Not applicable
