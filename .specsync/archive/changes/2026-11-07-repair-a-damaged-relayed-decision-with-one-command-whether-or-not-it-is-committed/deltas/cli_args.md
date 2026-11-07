## ADDED

### REQUIREMENT REQ-cli-args-relay-repair-001

The CLI SHALL parse `change repair-relays <id>` into the change action, taking the change id and no other arguments.

#### Acceptance Criteria

- Help lists `repair-relays` under `change`.
- `change repair-relays` without an id fails argument parsing, and existing change grammar parses exactly as before.
