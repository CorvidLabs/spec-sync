# Command registry truth delta

## ADDED

### REQUIREMENT REQ-commands-001

The system SHALL describe registered command modules using their current persisted layout and behavior.

Acceptance Criteria
- The init registry entry names the `.specsync/` 5.0 layout rather than the removed root JSON layout.
- Command documentation remains consistent with the dispatched modules.

## MODIFIED

### SPEC SECTION Purpose

Shared command infrastructure and registry used by all CLI subcommands. It centralizes config loading, spec discovery, filtering, schema construction, validation, exit handling, GitHub drift issues, and dispatch modules including the verified 5.0 change lifecycle.
