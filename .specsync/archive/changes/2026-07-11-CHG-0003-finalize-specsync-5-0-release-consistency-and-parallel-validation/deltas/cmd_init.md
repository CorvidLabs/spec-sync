# Initialization version-truth delta

## ADDED

### REQUIREMENT REQ-cmd-init-002

The system SHALL describe and create the same current versioned project layout.

Acceptance Criteria
- Canonical initialization documentation identifies the 5.0 layout and TOML configuration.
- Tests and examples do not describe the removed root JSON initialization path as current behavior.

## MODIFIED

### SPEC SECTION Purpose

Implements `specsync init`. Creates the 5.0 `.specsync/` layout with detected source directories, canonical TOML configuration, SDD policy, version stamp, local-state ignore rules, lifecycle/change/archive directories, and optional guided agent/change bootstrap.
