## ADDED

### REQUIREMENT REQ-cmd-change-042

`change ship-status` SHALL list the refusal of REQ-change-169 as a blocker, with the same message, and SHALL NOT report the change ready to finalize while it holds. `change ship --dry-run` SHALL refuse with that message.

Acceptance Criteria
- On the starling-shaped branch, `change ship-status <id>` lists the blocker, reports `ready_to_finalize` false and exits 1, and `change ship <id> --dry-run` refuses with the same message.
- With the lost blocks restored, `ready_to_finalize` is true again.
