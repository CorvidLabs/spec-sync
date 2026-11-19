## ADDED

### REQUIREMENT REQ-cli-args-026

The `history` subcommand SHALL take one required positional `<module>`, together with the global output flags.

Acceptance Criteria
- `specsync history auth` and `specsync --format json history auth` parse; `specsync history` with no module is refused by argument parsing.
