---
id: close-changes-that-will-not-ship
state: draft
type: feature
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# Close changes that will not ship

## Intent

Close changes that will not ship

## Affected Canonical Specs

- `change`
- `cmd_change`

## Acceptance Criteria

- change close moves each named draft, approved, implementing or verifying change into .specsync/archive/closed/<date>-<id>/ byte for byte with close.json (actor, reason, from_state, replaced_by, relay, reviewed false); status, show and ship-status name the close; list, check and new no longer use it; a batch with one bad id closes none; a change whose materialized spec text has not merged is refused naming what to restore or remove

## No-spec Rationale

Not applicable
