## ADDED

### REQUIREMENT REQ-change-052

The change module SHALL hold canonical ownership of its command-line wiring so
that acceptance inputs touching that wiring resolve to a single owning spec.

Acceptance Criteria

- `specs/change/change.spec.md` lists `src/commands/change.rs` among its files.
- A change whose acceptance inputs include `src/commands/change.rs` resolves
  deterministic canonical ownership at finalize.
- No other spec claims the same path, so ownership stays single-valued.
