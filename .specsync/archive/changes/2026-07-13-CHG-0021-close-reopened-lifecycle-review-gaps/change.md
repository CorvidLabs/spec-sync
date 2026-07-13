---
id: CHG-0021-close-reopened-lifecycle-review-gaps
state: archived
type: bug_fix
base_commit: 27dd307b84333905fbf8907a2c9082c27ebfb30d
---

# Close reopened lifecycle review gaps

## Intent

Close reopened lifecycle review gaps

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- Strict checks reject modified reopened definitions before closing; definition reapproval keeps reopened evidence in the verifying lane; nested project history fallback finds top-relative state paths; reopen rejects current delivery inputs even when another closing-validity condition fails

## No-spec Rationale

Not applicable
