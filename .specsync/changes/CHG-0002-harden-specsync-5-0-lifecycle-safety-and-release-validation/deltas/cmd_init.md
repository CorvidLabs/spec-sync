# Initialization hardening delta

## ADDED

### REQUIREMENT REQ-cmd-init-001

The system SHALL initialize local lifecycle coordination files as ignored, recoverable implementation details.

Acceptance Criteria
- New projects ignore the lifecycle lock and transaction journal.
- Initialization remains idempotent and does not weaken SDD enforcement.

## MODIFIED

### SPEC SECTION Purpose

Implements the `specsync init` command. Creates the 5.0 `.specsync/` layout with detected source directories, canonical configuration, SDD policy, version stamp, local-state ignore rules, lifecycle/change/archive directories, and optional guided agent/change bootstrap.

### SPEC SECTION Invariants

1. Auto-detects source directories via `config::detect_source_dirs()`.
2. Never overwrites an existing current or legacy configuration; legacy configurations receive a migration hint.
3. Writes the 5.0 policy, version, and layout deterministically without blocking in non-interactive environments.
4. Local hash cache, lifecycle lock, and transaction journal files are ignored and never treated as portable project state.
5. Re-running initialization is idempotent.
