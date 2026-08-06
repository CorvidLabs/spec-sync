## MODIFIED

### REQUIREMENT REQ-change-052

The change module SHALL hold canonical ownership of its own logic, leaving each
command wiring module the sole canonical owner of its own file.

Acceptance Criteria

- `specs/change/change.spec.md` lists `src/change.rs` and does not list
  `src/commands/change.rs`.
- `specs/cmd_change/cmd_change.spec.md` remains the sole claimant of
  `src/commands/change.rs`.
- No source file is claimed by two specs.
