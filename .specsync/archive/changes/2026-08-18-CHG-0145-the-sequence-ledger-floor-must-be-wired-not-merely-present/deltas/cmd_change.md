## ADDED

### REQUIREMENT REQ-cmd-change-013

The lifecycle staging path SHALL raise a stale sequence ledger before staging it, and that wiring SHALL be asserted rather than only the function that performs it.

Acceptance Criteria
- A commit produced by the lifecycle staging path over a working tree whose ledger is below the committed mark carries the committed mark, not the stale one.
- Removing the raise from the staging path fails a test, so the connection between the mechanism and its caller cannot be severed while the suite stays green.
