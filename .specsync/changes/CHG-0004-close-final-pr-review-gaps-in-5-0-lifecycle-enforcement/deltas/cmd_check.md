## ADDED

### REQUIREMENT REQ-cmd-check-001
Unified JSON checking SHALL preserve the documented top-level check schema when SDD validation fails.

Acceptance Criteria
- Failed SDD JSON output includes `passed`, `errors`, `warnings`, `stale`, and `specs_checked`.
- Structured SDD detail remains available as an additive field.
