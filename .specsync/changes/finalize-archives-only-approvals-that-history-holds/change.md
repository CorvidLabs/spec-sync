---
id: finalize-archives-only-approvals-that-history-holds
state: draft
type: feature
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# Finalize archives only approvals that history holds

## Intent

Finalize archives only approvals that history holds

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- finalize and ship refuse, writing nothing, an approvals.json edited since HEAD or never committed, naming the file and the restore; they refuse a committed version that rewrites an earlier approval, naming that commit; a committed relayed_by removal, as change repair-relays makes, still finalizes; an honest flow finalizes as before

## No-spec Rationale

Not applicable
