## ADDED

### REQUIREMENT REQ-cli-args-024

The `change status`, `change show`, `change list` and `change ship-status` subcommands SHALL accept an optional `--timeout <secs>` argument taking a whole number of seconds; a value that is not a non-negative integer SHALL be refused by argument parsing.

Acceptance Criteria
- `specsync change status --timeout 30` parses; `--timeout -1` and `--timeout abc` are refused.
