---
change: keep-check-fix-export-rows-in-one-contiguous-table-for-issue-768
artifact: requirements
---

# Requirements

### REQ-cmd-check-015
`check --fix` SHALL use one contiguous target table for both row width and insertion, prefer the first Name header, fall back to the first table.

Acceptance: preserve all surrounding content, pass strict recheck, and keep repeated fixes idempotent. These are proposed bug-specific criteria, not recorded human acceptance.
