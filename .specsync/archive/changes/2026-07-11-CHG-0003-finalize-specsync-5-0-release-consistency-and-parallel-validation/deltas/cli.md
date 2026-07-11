# CLI lifecycle truth delta

## ADDED

### REQUIREMENT REQ-cli-001

The system SHALL expose and document the verified SDD lifecycle through the root CLI dispatcher.

Acceptance Criteria
- The CLI contract lists the `change` namespace and current initialization layout.
- Dispatch documentation includes the change lifecycle handler.

## MODIFIED

### SPEC SECTION Purpose

The `specsync` command-line entry point parses global options, routes canonical validation and verified SDD lifecycle commands to focused handlers, and preserves equivalent human-readable and structured output without owning domain policy.
