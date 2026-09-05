---
id: allow-audited-reopening-when-legacy-acceptance-cannot-be-reconstructed
state: implementing
type: feature
base_commit: 6b1717038edb467d95bb483861f0c076da76deb5
---

# Allow audited reopening when legacy acceptance cannot be reconstructed

## Intent

Allow audited reopening when legacy acceptance cannot be reconstructed

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- Legacy accepted records without a manifest can reopen with explicit actor and reason when historical reconstruction fails despite current inputs and anchored evidence; the audit preserves prior evidence and records the cause; reconstructible legacy and current modern evidence still refuse; fresh verification and acceptance permit archival.

## No-spec Rationale

Not applicable
