## ADDED

### REQUIREMENT REQ-cli-args-011

The CLI SHALL expose `change ship-status [ID]` as a first-class `ChangeAction`.

Acceptance Criteria

- The subcommand is listed in `change --help`.
- With no ID, every active change is reported; with an ID, only that change is.
