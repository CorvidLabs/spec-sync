## ADDED

### REQUIREMENT REQ-cli-args-0069-001
The `cli_args` module SHALL retain deterministic canonical ownership of `src/cli.rs` for the two-verb lifecycle CLI surface through finalization.

Acceptance Criteria
- `src/cli.rs` has deterministic canonical ownership via `cli_args`.
- Check and audit verbs remain discoverable through the existing CLI surface.
