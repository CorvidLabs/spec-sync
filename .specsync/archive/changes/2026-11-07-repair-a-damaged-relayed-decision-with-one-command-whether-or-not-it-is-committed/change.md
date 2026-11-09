---
id: repair-a-damaged-relayed-decision-with-one-command-whether-or-not-it-is-committed
state: archived
type: bug_fix
base_commit: bc1859c4cfe1ee13fbfa54ad766a14fa7a2350c4
---

# Repair a damaged relayed decision with one command, whether or not it is committed

## Intent

Repair a damaged relayed decision with one command, whether or not it is committed

## Affected Canonical Specs

- `change`
- `cmd_change`
- `cli_args`

## Acceptance Criteria

- When one of those records is damaged, I am told one command that repairs it, whether or not it has been committed yet.
- That repair puts the original back when a copy of it still exists, and otherwise removes only the damaged record, never the approval or review it belongs to.
- The repair tells me what it put back and what it removed.

## No-spec Rationale

Not applicable
