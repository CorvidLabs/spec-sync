# CLI argument lifecycle truth delta

## ADDED

### REQUIREMENT REQ-cli-args-001

The system SHALL declare the complete verified SDD change command grammar in the shared Clap parser.

Acceptance Criteria
- `Command` includes the `Change` namespace.
- `ChangeAction` declares every lifecycle, inspection, checking, and adoption operation.

## MODIFIED

### SPEC SECTION Purpose

Defines the complete CLI argument grammar using Clap derive macros, including global options, canonical spec commands, agent integration, and the verified SDD `change` namespace.
