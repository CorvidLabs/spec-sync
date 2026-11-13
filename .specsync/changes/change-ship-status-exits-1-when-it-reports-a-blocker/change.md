---
id: change-ship-status-exits-1-when-it-reports-a-blocker
state: approved
type: bug_fix
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# Change ship-status exits 1 when it reports a blocker

## Intent

change ship-status exits 1 when it reports a blocker

## Affected Canonical Specs

- `cmd_change`

## Acceptance Criteria

- an approved, unverified change: change ship-status <id> prints its Blocker line and exits 1, and --json prints one document with a non-empty blockers list and exits 1; change ship-status with no id exits 1 when any listed change has a blocker; a change ready to finalize and a closed-late change exit 0 as before

## No-spec Rationale

Not applicable
