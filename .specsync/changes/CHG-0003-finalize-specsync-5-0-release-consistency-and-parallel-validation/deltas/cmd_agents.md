# Agent command dispatcher truth delta

## ADDED

### REQUIREMENT REQ-cmd-agents-001

The system SHALL dispatch installation of the complete current native agent artifact set.

Acceptance Criteria
- The dispatcher routes all four agent targets without changing artifact semantics.
- Canonical context names both create-spec and create-change where supported.

## MODIFIED

### SPEC SECTION Purpose

Implements `specsync agents` by routing install, uninstall, and status actions for the project-local verified-SDD skills and supported create-spec/create-change commands.
