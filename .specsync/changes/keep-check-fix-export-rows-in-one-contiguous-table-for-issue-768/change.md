---
id: keep-check-fix-export-rows-in-one-contiguous-table-for-issue-768
state: implementing
type: bug_fix
base_commit: 9003fad7ec4dea3938179d076248b48f29b6ed48
---

# Keep check fix export rows in one contiguous table for issue 768

## Intent

Keep check fix export rows in one contiguous table for issue 768

## Affected Canonical Specs

- `cmd_check`

## Acceptance Criteria

- For issue 768 case 1, new export rows extend one contiguous table using its own width; prefer the first Name header and otherwise the first table; preserve surrounding content; pass strict recheck and repeated-fix idempotence. Case 2 remains out of scope.

## No-spec Rationale

Not applicable
