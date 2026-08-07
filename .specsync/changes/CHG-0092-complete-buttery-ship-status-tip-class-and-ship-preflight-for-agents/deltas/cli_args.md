## MODIFIED

### REQUIREMENT REQ-cli-args-011

The CLI SHALL expose `change ship-status [ID]` and `change ship [ID]` as
first-class `ChangeAction`s.

Acceptance Criteria

- Both subcommands are listed in `change --help`.
- `ship-status --help` mentions tip class, trust guidance, archive, and finalize.
- `ship --help` describes preflight and finalize-when-ready behavior.
- With no ID, `ship-status` reports every active change; `ship` requires a change
  id or a unique active change.

## ADDED

### REQUIREMENT REQ-cli-args-012

`change finalize --help` SHALL mention that finalization writes an archive tip
intended for the same pull request, and that merging before finalize orphans
verification evidence.

Acceptance Criteria

- Help text for `change finalize` references the archive tip and same-PR merge
  order (merge only after finalize).
