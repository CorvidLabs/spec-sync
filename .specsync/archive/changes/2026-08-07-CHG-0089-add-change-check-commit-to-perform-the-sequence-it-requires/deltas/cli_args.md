## ADDED

### REQUIREMENT REQ-cli-args-010

The CLI argument surface for `change check` SHALL accept `--commit` and `--push`.
`--push` requires `--commit`.

Acceptance Criteria

- Parsing `change check --commit --push` sets both flags.
- Parsing bare `change check` leaves both flags false.
- Help text exposes both flags.
